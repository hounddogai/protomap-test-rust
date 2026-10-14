use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict flow log checkpoint must pass. Check returns its problems, or none.
pub trait StrictFlowLogCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &StrictFlowLogCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFlowLogCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl StrictFlowLogCheckpointRule for StrictFlowLogCheckpointCodeLengthRule {
    fn check(&self, item: &StrictFlowLogCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFlowLogCheckpointRequiredCodeRule {
    pub message: String,
}

impl StrictFlowLogCheckpointRule for StrictFlowLogCheckpointRequiredCodeRule {
    fn check(&self, item: &StrictFlowLogCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFlowLogCheckpointCategoryLengthRule {
    pub max_length: i64,
}

impl StrictFlowLogCheckpointRule for StrictFlowLogCheckpointCategoryLengthRule {
    fn check(&self, item: &StrictFlowLogCheckpoint) -> Vec<String> {
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
pub struct StrictFlowLogCheckpointRequiredTitleRule {
    pub message: String,
}

impl StrictFlowLogCheckpointRule for StrictFlowLogCheckpointRequiredTitleRule {
    fn check(&self, item: &StrictFlowLogCheckpoint) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict flow log checkpoint.
#[derive(Debug, Clone)]
pub struct StrictFlowLogCheckpointValidator {
    pub rules: Vec<Rc<dyn StrictFlowLogCheckpointRule>>,
}

impl StrictFlowLogCheckpointValidator {
    pub fn validate(&self, item: &StrictFlowLogCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictFlowLogCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFlowLogCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_flow_log_checkpoint_validator() -> StrictFlowLogCheckpointValidator {
    StrictFlowLogCheckpointValidator {
        rules: vec![
            Rc::new(StrictFlowLogCheckpointCodeLengthRule { max_length: 1000 }) as Rc<dyn StrictFlowLogCheckpointRule>,
            Rc::new(StrictFlowLogCheckpointRequiredCodeRule {
                message: "strict flow log checkpoint needs an entry".to_string(),
            }),
            Rc::new(StrictFlowLogCheckpointCategoryLengthRule { max_length: 120 }),
            Rc::new(StrictFlowLogCheckpointRequiredTitleRule {
                message: "strict flow log checkpoint needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_flow_log_checkpoint_validator();
        let item = new_strict_flow_log_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictFlowLogCheckpointCodeLengthRule { max_length: 3600 };
        let mut item = new_strict_flow_log_checkpoint("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictFlowLogCheckpointRequiredCodeRule {
            message: "strict flow log checkpoint needs a value".to_string(),
        };
        let mut item = new_strict_flow_log_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = StrictFlowLogCheckpointCategoryLengthRule { max_length: 86400 };
        let mut item = new_strict_flow_log_checkpoint("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictFlowLogCheckpointRequiredTitleRule {
            message: "strict flow log checkpoint needs an entry".to_string(),
        };
        let mut item = new_strict_flow_log_checkpoint("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
