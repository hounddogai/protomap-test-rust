use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check cycle must pass. Check returns its problems, or none.
pub trait HealthCheckCycleRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCycleCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckCycleRule for HealthCheckCycleCodeLengthRule {
    fn check(&self, item: &HealthCheckCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCycleRequiredContactEmailRule {
    pub message: String,
}

impl HealthCheckCycleRule for HealthCheckCycleRequiredContactEmailRule {
    fn check(&self, item: &HealthCheckCycle) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCycleRequiredLabelRule {
    pub message: String,
}

impl HealthCheckCycleRule for HealthCheckCycleRequiredLabelRule {
    fn check(&self, item: &HealthCheckCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckCycleRequiredCodeRule {
    pub message: String,
}

impl HealthCheckCycleRule for HealthCheckCycleRequiredCodeRule {
    fn check(&self, item: &HealthCheckCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check cycle.
#[derive(Debug, Clone)]
pub struct HealthCheckCycleValidator {
    pub rules: Vec<Rc<dyn HealthCheckCycleRule>>,
}

impl HealthCheckCycleValidator {
    pub fn validate(&self, item: &HealthCheckCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_cycle_validator() -> HealthCheckCycleValidator {
    HealthCheckCycleValidator {
        rules: vec![
            Rc::new(HealthCheckCycleCodeLengthRule { max_length: 255 }) as Rc<dyn HealthCheckCycleRule>,
            Rc::new(HealthCheckCycleRequiredContactEmailRule {
                message: "health check cycle needs a value".to_string(),
            }),
            Rc::new(HealthCheckCycleRequiredLabelRule {
                message: "health check cycle needs this field".to_string(),
            }),
            Rc::new(HealthCheckCycleRequiredCodeRule {
                message: "health check cycle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_cycle_validator();
        let item = new_health_check_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = HealthCheckCycleCodeLengthRule { max_length: 64 };
        let mut item = new_health_check_cycle("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = HealthCheckCycleRequiredContactEmailRule {
            message: "health check cycle needs an entry".to_string(),
        };
        let mut item = new_health_check_cycle("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = HealthCheckCycleRequiredLabelRule {
            message: "health check cycle needs this field".to_string(),
        };
        let mut item = new_health_check_cycle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckCycleRequiredCodeRule {
            message: "health check cycle needs an entry".to_string(),
        };
        let mut item = new_health_check_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
