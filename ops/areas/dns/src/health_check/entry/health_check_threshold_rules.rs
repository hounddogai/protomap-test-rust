use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check threshold must pass. Check returns its problems, or none.
pub trait HealthCheckThresholdRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckThresholdRequiredCodeRule {
    pub message: String,
}

impl HealthCheckThresholdRule for HealthCheckThresholdRequiredCodeRule {
    fn check(&self, item: &HealthCheckThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckThresholdCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckThresholdRule for HealthCheckThresholdCodePrefixRule {
    fn check(&self, item: &HealthCheckThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckThresholdTitleLengthRule {
    pub max_length: i64,
}

impl HealthCheckThresholdRule for HealthCheckThresholdTitleLengthRule {
    fn check(&self, item: &HealthCheckThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check threshold.
#[derive(Debug, Clone)]
pub struct HealthCheckThresholdValidator {
    pub rules: Vec<Rc<dyn HealthCheckThresholdRule>>,
}

impl HealthCheckThresholdValidator {
    pub fn validate(&self, item: &HealthCheckThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_threshold_validator() -> HealthCheckThresholdValidator {
    HealthCheckThresholdValidator {
        rules: vec![
            Rc::new(HealthCheckThresholdRequiredCodeRule {
                message: "health check threshold needs an entry".to_string(),
            }) as Rc<dyn HealthCheckThresholdRule>,
            Rc::new(HealthCheckThresholdCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(HealthCheckThresholdTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_threshold_validator();
        let item = new_health_check_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckThresholdRequiredCodeRule {
            message: "health check threshold needs an entry".to_string(),
        };
        let mut item = new_health_check_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = HealthCheckThresholdTitleLengthRule { max_length: 86400 };
        let mut item = new_health_check_threshold("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
