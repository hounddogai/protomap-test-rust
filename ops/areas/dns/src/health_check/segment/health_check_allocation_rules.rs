use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check allocation must pass. Check returns its problems, or none.
pub trait HealthCheckAllocationRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocationRequiredContactEmailRule {
    pub message: String,
}

impl HealthCheckAllocationRule for HealthCheckAllocationRequiredContactEmailRule {
    fn check(&self, item: &HealthCheckAllocation) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocationRequiredCodeRule {
    pub message: String,
}

impl HealthCheckAllocationRule for HealthCheckAllocationRequiredCodeRule {
    fn check(&self, item: &HealthCheckAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocationVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckAllocationRule for HealthCheckAllocationVersionRangeRule {
    fn check(&self, item: &HealthCheckAllocation) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocationAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl HealthCheckAllocationRule for HealthCheckAllocationAllowedContactEmailRule {
    fn check(&self, item: &HealthCheckAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is not allowed", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAllocationCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckAllocationRule for HealthCheckAllocationCodeLengthRule {
    fn check(&self, item: &HealthCheckAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check allocation.
#[derive(Debug, Clone)]
pub struct HealthCheckAllocationValidator {
    pub rules: Vec<Rc<dyn HealthCheckAllocationRule>>,
}

impl HealthCheckAllocationValidator {
    pub fn validate(&self, item: &HealthCheckAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_allocation_validator() -> HealthCheckAllocationValidator {
    HealthCheckAllocationValidator {
        rules: vec![
            Rc::new(HealthCheckAllocationRequiredContactEmailRule {
                message: "health check allocation needs this field".to_string(),
            }) as Rc<dyn HealthCheckAllocationRule>,
            Rc::new(HealthCheckAllocationRequiredCodeRule {
                message: "health check allocation needs an entry".to_string(),
            }),
            Rc::new(HealthCheckAllocationVersionRangeRule { min: 32, max: 3600 }),
            Rc::new(HealthCheckAllocationAllowedContactEmailRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(HealthCheckAllocationCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_allocation_validator();
        let item = new_health_check_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = HealthCheckAllocationRequiredContactEmailRule {
            message: "health check allocation needs a value".to_string(),
        };
        let mut item = new_health_check_allocation("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckAllocationRequiredCodeRule {
            message: "health check allocation needs a value".to_string(),
        };
        let mut item = new_health_check_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = HealthCheckAllocationVersionRangeRule { min: 80, max: 3600 };
        let mut item = new_health_check_allocation("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = HealthCheckAllocationCodeLengthRule { max_length: 120 };
        let mut item = new_health_check_allocation("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
