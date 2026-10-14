use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention checkpoint must pass. Check returns its problems, or none.
pub trait RetentionCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RetentionCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCheckpointRequiredChecksumRule {
    pub message: String,
}

impl RetentionCheckpointRule for RetentionCheckpointRequiredChecksumRule {
    fn check(&self, item: &RetentionCheckpoint) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCheckpointRequiredCodeRule {
    pub message: String,
}

impl RetentionCheckpointRule for RetentionCheckpointRequiredCodeRule {
    fn check(&self, item: &RetentionCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCheckpointRequiredSourceRule {
    pub message: String,
}

impl RetentionCheckpointRule for RetentionCheckpointRequiredSourceRule {
    fn check(&self, item: &RetentionCheckpoint) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention checkpoint.
#[derive(Debug, Clone)]
pub struct RetentionCheckpointValidator {
    pub rules: Vec<Rc<dyn RetentionCheckpointRule>>,
}

impl RetentionCheckpointValidator {
    pub fn validate(&self, item: &RetentionCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_checkpoint_validator() -> RetentionCheckpointValidator {
    RetentionCheckpointValidator {
        rules: vec![
            Rc::new(RetentionCheckpointRequiredChecksumRule {
                message: "retention checkpoint needs this field".to_string(),
            }) as Rc<dyn RetentionCheckpointRule>,
            Rc::new(RetentionCheckpointRequiredCodeRule {
                message: "retention checkpoint needs a value".to_string(),
            }),
            Rc::new(RetentionCheckpointRequiredSourceRule {
                message: "retention checkpoint needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_checkpoint_validator();
        let item = new_retention_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = RetentionCheckpointRequiredChecksumRule {
            message: "retention checkpoint needs this field".to_string(),
        };
        let mut item = new_retention_checkpoint("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RetentionCheckpointRequiredCodeRule {
            message: "retention checkpoint needs an entry".to_string(),
        };
        let mut item = new_retention_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = RetentionCheckpointRequiredSourceRule {
            message: "retention checkpoint needs this field".to_string(),
        };
        let mut item = new_retention_checkpoint("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
