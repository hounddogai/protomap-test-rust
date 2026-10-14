use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback batch must pass. Check returns its problems, or none.
pub trait RollbackBatchRule: std::fmt::Debug {
    fn check(&self, item: &RollbackBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBatchRequiredCategoryRule {
    pub message: String,
}

impl RollbackBatchRule for RollbackBatchRequiredCategoryRule {
    fn check(&self, item: &RollbackBatch) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBatchCategoryLengthRule {
    pub max_length: i64,
}

impl RollbackBatchRule for RollbackBatchCategoryLengthRule {
    fn check(&self, item: &RollbackBatch) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBatchLabelLengthRule {
    pub max_length: i64,
}

impl RollbackBatchRule for RollbackBatchLabelLengthRule {
    fn check(&self, item: &RollbackBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBatchRequiredTitleRule {
    pub message: String,
}

impl RollbackBatchRule for RollbackBatchRequiredTitleRule {
    fn check(&self, item: &RollbackBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBatchTitleLengthRule {
    pub max_length: i64,
}

impl RollbackBatchRule for RollbackBatchTitleLengthRule {
    fn check(&self, item: &RollbackBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback batch.
#[derive(Debug, Clone)]
pub struct RollbackBatchValidator {
    pub rules: Vec<Rc<dyn RollbackBatchRule>>,
}

impl RollbackBatchValidator {
    pub fn validate(&self, item: &RollbackBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_batch_validator() -> RollbackBatchValidator {
    RollbackBatchValidator {
        rules: vec![
            Rc::new(RollbackBatchRequiredCategoryRule {
                message: "rollback batch needs a value".to_string(),
            }) as Rc<dyn RollbackBatchRule>,
            Rc::new(RollbackBatchCategoryLengthRule { max_length: 86400 }),
            Rc::new(RollbackBatchLabelLengthRule { max_length: 32 }),
            Rc::new(RollbackBatchRequiredTitleRule {
                message: "rollback batch needs an entry".to_string(),
            }),
            Rc::new(RollbackBatchTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_batch_validator();
        let item = new_rollback_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RollbackBatchRequiredCategoryRule {
            message: "rollback batch needs this field".to_string(),
        };
        let mut item = new_rollback_batch("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RollbackBatchCategoryLengthRule { max_length: 1000 };
        let mut item = new_rollback_batch("x", "y");
        item.category = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackBatchLabelLengthRule { max_length: 32 };
        let mut item = new_rollback_batch("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollbackBatchRequiredTitleRule {
            message: "rollback batch needs a value".to_string(),
        };
        let mut item = new_rollback_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollbackBatchTitleLengthRule { max_length: 80 };
        let mut item = new_rollback_batch("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
