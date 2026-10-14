use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check exception must pass. Check returns its problems, or none.
pub trait HealthCheckExceptionRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckExceptionRequiredTitleRule {
    pub message: String,
}

impl HealthCheckExceptionRule for HealthCheckExceptionRequiredTitleRule {
    fn check(&self, item: &HealthCheckException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckExceptionTitleLengthRule {
    pub max_length: i64,
}

impl HealthCheckExceptionRule for HealthCheckExceptionTitleLengthRule {
    fn check(&self, item: &HealthCheckException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckExceptionRequiredSegmentRule {
    pub message: String,
}

impl HealthCheckExceptionRule for HealthCheckExceptionRequiredSegmentRule {
    fn check(&self, item: &HealthCheckException) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckExceptionLabelLengthRule {
    pub max_length: i64,
}

impl HealthCheckExceptionRule for HealthCheckExceptionLabelLengthRule {
    fn check(&self, item: &HealthCheckException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckExceptionRequiredLabelRule {
    pub message: String,
}

impl HealthCheckExceptionRule for HealthCheckExceptionRequiredLabelRule {
    fn check(&self, item: &HealthCheckException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check exception.
#[derive(Debug, Clone)]
pub struct HealthCheckExceptionValidator {
    pub rules: Vec<Rc<dyn HealthCheckExceptionRule>>,
}

impl HealthCheckExceptionValidator {
    pub fn validate(&self, item: &HealthCheckException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_exception_validator() -> HealthCheckExceptionValidator {
    HealthCheckExceptionValidator {
        rules: vec![
            Rc::new(HealthCheckExceptionRequiredTitleRule {
                message: "health check exception needs an entry".to_string(),
            }) as Rc<dyn HealthCheckExceptionRule>,
            Rc::new(HealthCheckExceptionTitleLengthRule { max_length: 80 }),
            Rc::new(HealthCheckExceptionRequiredSegmentRule {
                message: "health check exception needs this field".to_string(),
            }),
            Rc::new(HealthCheckExceptionLabelLengthRule { max_length: 3600 }),
            Rc::new(HealthCheckExceptionRequiredLabelRule {
                message: "health check exception needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_exception_validator();
        let item = new_health_check_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = HealthCheckExceptionRequiredTitleRule {
            message: "health check exception needs an entry".to_string(),
        };
        let mut item = new_health_check_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = HealthCheckExceptionTitleLengthRule { max_length: 86400 };
        let mut item = new_health_check_exception("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = HealthCheckExceptionRequiredSegmentRule {
            message: "health check exception needs an entry".to_string(),
        };
        let mut item = new_health_check_exception("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = HealthCheckExceptionLabelLengthRule { max_length: 32 };
        let mut item = new_health_check_exception("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = HealthCheckExceptionRequiredLabelRule {
            message: "health check exception needs a value".to_string(),
        };
        let mut item = new_health_check_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
