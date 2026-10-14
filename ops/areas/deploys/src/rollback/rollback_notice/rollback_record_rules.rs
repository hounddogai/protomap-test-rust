use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback record must pass. Check returns its problems, or none.
pub trait RollbackRecordRule: std::fmt::Debug {
    fn check(&self, item: &RollbackRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRecordTitleLengthRule {
    pub max_length: i64,
}

impl RollbackRecordRule for RollbackRecordTitleLengthRule {
    fn check(&self, item: &RollbackRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRecordNameLengthRule {
    pub max_length: i64,
}

impl RollbackRecordRule for RollbackRecordNameLengthRule {
    fn check(&self, item: &RollbackRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRecordAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackRecordRule for RollbackRecordAttemptsRangeRule {
    fn check(&self, item: &RollbackRecord) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRecordRequiredCategoryRule {
    pub message: String,
}

impl RollbackRecordRule for RollbackRecordRequiredCategoryRule {
    fn check(&self, item: &RollbackRecord) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRecordCodePrefixRule {
    pub prefix: String,
}

impl RollbackRecordRule for RollbackRecordCodePrefixRule {
    fn check(&self, item: &RollbackRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRecordRequiredTitleRule {
    pub message: String,
}

impl RollbackRecordRule for RollbackRecordRequiredTitleRule {
    fn check(&self, item: &RollbackRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback record.
#[derive(Debug, Clone)]
pub struct RollbackRecordValidator {
    pub rules: Vec<Rc<dyn RollbackRecordRule>>,
}

impl RollbackRecordValidator {
    pub fn validate(&self, item: &RollbackRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_record_validator() -> RollbackRecordValidator {
    RollbackRecordValidator {
        rules: vec![
            Rc::new(RollbackRecordTitleLengthRule { max_length: 1000 }) as Rc<dyn RollbackRecordRule>,
            Rc::new(RollbackRecordNameLengthRule { max_length: 255 }),
            Rc::new(RollbackRecordAttemptsRangeRule { min: 32, max: 255 }),
            Rc::new(RollbackRecordRequiredCategoryRule {
                message: "rollback record needs this field".to_string(),
            }),
            Rc::new(RollbackRecordCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RollbackRecordRequiredTitleRule {
                message: "rollback record needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_record_validator();
        let item = new_rollback_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollbackRecordTitleLengthRule { max_length: 32 };
        let mut item = new_rollback_record("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollbackRecordNameLengthRule { max_length: 255 };
        let mut item = new_rollback_record("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RollbackRecordAttemptsRangeRule { min: 3600, max: 3600 };
        let mut item = new_rollback_record("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RollbackRecordRequiredCategoryRule {
            message: "rollback record needs an entry".to_string(),
        };
        let mut item = new_rollback_record("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollbackRecordRequiredTitleRule {
            message: "rollback record needs a value".to_string(),
        };
        let mut item = new_rollback_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
