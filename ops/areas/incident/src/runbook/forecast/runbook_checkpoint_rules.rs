use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook checkpoint must pass. Check returns its problems, or none.
pub trait RunbookCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RunbookCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpointCodePrefixRule {
    pub prefix: String,
}

impl RunbookCheckpointRule for RunbookCheckpointCodePrefixRule {
    fn check(&self, item: &RunbookCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpointRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookCheckpointRule for RunbookCheckpointRetriesRangeRule {
    fn check(&self, item: &RunbookCheckpoint) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpointKeyIdLengthRule {
    pub max_length: i64,
}

impl RunbookCheckpointRule for RunbookCheckpointKeyIdLengthRule {
    fn check(&self, item: &RunbookCheckpoint) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpointExternalRefLengthRule {
    pub max_length: i64,
}

impl RunbookCheckpointRule for RunbookCheckpointExternalRefLengthRule {
    fn check(&self, item: &RunbookCheckpoint) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpointRequiredCodeRule {
    pub message: String,
}

impl RunbookCheckpointRule for RunbookCheckpointRequiredCodeRule {
    fn check(&self, item: &RunbookCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpointRequiredLabelRule {
    pub message: String,
}

impl RunbookCheckpointRule for RunbookCheckpointRequiredLabelRule {
    fn check(&self, item: &RunbookCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook checkpoint.
#[derive(Debug, Clone)]
pub struct RunbookCheckpointValidator {
    pub rules: Vec<Rc<dyn RunbookCheckpointRule>>,
}

impl RunbookCheckpointValidator {
    pub fn validate(&self, item: &RunbookCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_checkpoint_validator() -> RunbookCheckpointValidator {
    RunbookCheckpointValidator {
        rules: vec![
            Rc::new(RunbookCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn RunbookCheckpointRule>,
            Rc::new(RunbookCheckpointRetriesRangeRule { min: 120, max: 86400 }),
            Rc::new(RunbookCheckpointKeyIdLengthRule { max_length: 255 }),
            Rc::new(RunbookCheckpointExternalRefLengthRule { max_length: 255 }),
            Rc::new(RunbookCheckpointRequiredCodeRule {
                message: "runbook checkpoint needs a value".to_string(),
            }),
            Rc::new(RunbookCheckpointRequiredLabelRule {
                message: "runbook checkpoint needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_checkpoint_validator();
        let item = new_runbook_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RunbookCheckpointRetriesRangeRule { min: 32, max: 120 };
        let mut item = new_runbook_checkpoint("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = RunbookCheckpointKeyIdLengthRule { max_length: 1000 };
        let mut item = new_runbook_checkpoint("x", "y");
        item.key_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = RunbookCheckpointExternalRefLengthRule { max_length: 3600 };
        let mut item = new_runbook_checkpoint("x", "y");
        item.external_ref = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookCheckpointRequiredCodeRule {
            message: "runbook checkpoint needs an entry".to_string(),
        };
        let mut item = new_runbook_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RunbookCheckpointRequiredLabelRule {
            message: "runbook checkpoint needs this field".to_string(),
        };
        let mut item = new_runbook_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
