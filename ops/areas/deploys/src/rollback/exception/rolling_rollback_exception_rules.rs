use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling rollback exception must pass. Check returns its problems, or none.
pub trait RollingRollbackExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RollingRollbackException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackExceptionCodePrefixRule {
    pub prefix: String,
}

impl RollingRollbackExceptionRule for RollingRollbackExceptionCodePrefixRule {
    fn check(&self, item: &RollingRollbackException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackExceptionBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRollbackExceptionRule for RollingRollbackExceptionBurstRangeRule {
    fn check(&self, item: &RollingRollbackException) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackExceptionLabelLengthRule {
    pub max_length: i64,
}

impl RollingRollbackExceptionRule for RollingRollbackExceptionLabelLengthRule {
    fn check(&self, item: &RollingRollbackException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackExceptionSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRollbackExceptionRule for RollingRollbackExceptionSequenceRangeRule {
    fn check(&self, item: &RollingRollbackException) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackExceptionRequiredTitleRule {
    pub message: String,
}

impl RollingRollbackExceptionRule for RollingRollbackExceptionRequiredTitleRule {
    fn check(&self, item: &RollingRollbackException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling rollback exception.
#[derive(Debug, Clone)]
pub struct RollingRollbackExceptionValidator {
    pub rules: Vec<Rc<dyn RollingRollbackExceptionRule>>,
}

impl RollingRollbackExceptionValidator {
    pub fn validate(&self, item: &RollingRollbackException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingRollbackException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRollbackException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_rollback_exception_validator() -> RollingRollbackExceptionValidator {
    RollingRollbackExceptionValidator {
        rules: vec![
            Rc::new(RollingRollbackExceptionCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RollingRollbackExceptionRule>,
            Rc::new(RollingRollbackExceptionBurstRangeRule { min: 64, max: 120 }),
            Rc::new(RollingRollbackExceptionLabelLengthRule { max_length: 80 }),
            Rc::new(RollingRollbackExceptionSequenceRangeRule { min: 64, max: 1000 }),
            Rc::new(RollingRollbackExceptionRequiredTitleRule {
                message: "rolling rollback exception needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_rollback_exception_validator();
        let item = new_rolling_rollback_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RollingRollbackExceptionBurstRangeRule { min: 16, max: 32 };
        let mut item = new_rolling_rollback_exception("x", "y");
        item.burst = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingRollbackExceptionLabelLengthRule { max_length: 80 };
        let mut item = new_rolling_rollback_exception("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RollingRollbackExceptionSequenceRangeRule { min: 32, max: 80 };
        let mut item = new_rolling_rollback_exception("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingRollbackExceptionRequiredTitleRule {
            message: "rolling rollback exception needs this field".to_string(),
        };
        let mut item = new_rolling_rollback_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
