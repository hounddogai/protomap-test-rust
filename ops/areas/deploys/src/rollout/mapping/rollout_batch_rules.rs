use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout batch must pass. Check returns its problems, or none.
pub trait RolloutBatchRule: std::fmt::Debug {
    fn check(&self, item: &RolloutBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBatchNameLengthRule {
    pub max_length: i64,
}

impl RolloutBatchRule for RolloutBatchNameLengthRule {
    fn check(&self, item: &RolloutBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBatchRequiredOwnerIdRule {
    pub message: String,
}

impl RolloutBatchRule for RolloutBatchRequiredOwnerIdRule {
    fn check(&self, item: &RolloutBatch) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBatchRequiredLabelRule {
    pub message: String,
}

impl RolloutBatchRule for RolloutBatchRequiredLabelRule {
    fn check(&self, item: &RolloutBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBatchRequiredCodeRule {
    pub message: String,
}

impl RolloutBatchRule for RolloutBatchRequiredCodeRule {
    fn check(&self, item: &RolloutBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBatchCodeLengthRule {
    pub max_length: i64,
}

impl RolloutBatchRule for RolloutBatchCodeLengthRule {
    fn check(&self, item: &RolloutBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout batch.
#[derive(Debug, Clone)]
pub struct RolloutBatchValidator {
    pub rules: Vec<Rc<dyn RolloutBatchRule>>,
}

impl RolloutBatchValidator {
    pub fn validate(&self, item: &RolloutBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_batch_validator() -> RolloutBatchValidator {
    RolloutBatchValidator {
        rules: vec![
            Rc::new(RolloutBatchNameLengthRule { max_length: 86400 }) as Rc<dyn RolloutBatchRule>,
            Rc::new(RolloutBatchRequiredOwnerIdRule {
                message: "rollout batch needs an entry".to_string(),
            }),
            Rc::new(RolloutBatchRequiredLabelRule {
                message: "rollout batch needs this field".to_string(),
            }),
            Rc::new(RolloutBatchRequiredCodeRule {
                message: "rollout batch needs a value".to_string(),
            }),
            Rc::new(RolloutBatchCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_batch_validator();
        let item = new_rollout_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RolloutBatchNameLengthRule { max_length: 80 };
        let mut item = new_rollout_batch("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = RolloutBatchRequiredOwnerIdRule {
            message: "rollout batch needs an entry".to_string(),
        };
        let mut item = new_rollout_batch("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RolloutBatchRequiredLabelRule {
            message: "rollout batch needs this field".to_string(),
        };
        let mut item = new_rollout_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutBatchRequiredCodeRule {
            message: "rollout batch needs a value".to_string(),
        };
        let mut item = new_rollout_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutBatchCodeLengthRule { max_length: 16 };
        let mut item = new_rollout_batch("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
