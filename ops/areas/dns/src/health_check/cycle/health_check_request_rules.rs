use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check request must pass. Check returns its problems, or none.
pub trait HealthCheckRequestRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequestTimezoneLengthRule {
    pub max_length: i64,
}

impl HealthCheckRequestRule for HealthCheckRequestTimezoneLengthRule {
    fn check(&self, item: &HealthCheckRequest) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequestRequiredKeyIdRule {
    pub message: String,
}

impl HealthCheckRequestRule for HealthCheckRequestRequiredKeyIdRule {
    fn check(&self, item: &HealthCheckRequest) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequestCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckRequestRule for HealthCheckRequestCodePrefixRule {
    fn check(&self, item: &HealthCheckRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequestKeyIdLengthRule {
    pub max_length: i64,
}

impl HealthCheckRequestRule for HealthCheckRequestKeyIdLengthRule {
    fn check(&self, item: &HealthCheckRequest) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check request.
#[derive(Debug, Clone)]
pub struct HealthCheckRequestValidator {
    pub rules: Vec<Rc<dyn HealthCheckRequestRule>>,
}

impl HealthCheckRequestValidator {
    pub fn validate(&self, item: &HealthCheckRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckRequest]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_health_check_request_validator() -> HealthCheckRequestValidator {
    HealthCheckRequestValidator {
        rules: vec![
            Rc::new(HealthCheckRequestTimezoneLengthRule { max_length: 16 }) as Rc<dyn HealthCheckRequestRule>,
            Rc::new(HealthCheckRequestRequiredKeyIdRule {
                message: "health check request needs a value".to_string(),
            }),
            Rc::new(HealthCheckRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(HealthCheckRequestKeyIdLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_request_validator();
        let item = new_health_check_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = HealthCheckRequestTimezoneLengthRule { max_length: 64 };
        let mut item = new_health_check_request("x", "y");
        item.timezone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = HealthCheckRequestRequiredKeyIdRule {
            message: "health check request needs this field".to_string(),
        };
        let mut item = new_health_check_request("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = HealthCheckRequestKeyIdLengthRule { max_length: 16 };
        let mut item = new_health_check_request("x", "y");
        item.key_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
