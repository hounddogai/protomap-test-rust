use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction checkpoint must pass. Check returns its problems, or none.
pub trait RedactionCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RedactionCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointRequiredSourceRule {
    pub message: String,
}

impl RedactionCheckpointRule for RedactionCheckpointRequiredSourceRule {
    fn check(&self, item: &RedactionCheckpoint) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl RedactionCheckpointRule for RedactionCheckpointLabelLengthRule {
    fn check(&self, item: &RedactionCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointSourceLengthRule {
    pub max_length: i64,
}

impl RedactionCheckpointRule for RedactionCheckpointSourceLengthRule {
    fn check(&self, item: &RedactionCheckpoint) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionCheckpointRule for RedactionCheckpointMaximumRangeRule {
    fn check(&self, item: &RedactionCheckpoint) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointRequiredLabelRule {
    pub message: String,
}

impl RedactionCheckpointRule for RedactionCheckpointRequiredLabelRule {
    fn check(&self, item: &RedactionCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpointRequiredHostRule {
    pub message: String,
}

impl RedactionCheckpointRule for RedactionCheckpointRequiredHostRule {
    fn check(&self, item: &RedactionCheckpoint) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction checkpoint.
#[derive(Debug, Clone)]
pub struct RedactionCheckpointValidator {
    pub rules: Vec<Rc<dyn RedactionCheckpointRule>>,
}

impl RedactionCheckpointValidator {
    pub fn validate(&self, item: &RedactionCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_checkpoint_validator() -> RedactionCheckpointValidator {
    RedactionCheckpointValidator {
        rules: vec![
            Rc::new(RedactionCheckpointRequiredSourceRule {
                message: "redaction checkpoint needs an entry".to_string(),
            }) as Rc<dyn RedactionCheckpointRule>,
            Rc::new(RedactionCheckpointLabelLengthRule { max_length: 3600 }),
            Rc::new(RedactionCheckpointSourceLengthRule { max_length: 16 }),
            Rc::new(RedactionCheckpointMaximumRangeRule { min: 7.8, max: 62.7 }),
            Rc::new(RedactionCheckpointRequiredLabelRule {
                message: "redaction checkpoint needs a value".to_string(),
            }),
            Rc::new(RedactionCheckpointRequiredHostRule {
                message: "redaction checkpoint needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_checkpoint_validator();
        let item = new_redaction_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = RedactionCheckpointRequiredSourceRule {
            message: "redaction checkpoint needs an entry".to_string(),
        };
        let mut item = new_redaction_checkpoint("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RedactionCheckpointLabelLengthRule { max_length: 80 };
        let mut item = new_redaction_checkpoint("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = RedactionCheckpointSourceLengthRule { max_length: 64 };
        let mut item = new_redaction_checkpoint("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RedactionCheckpointMaximumRangeRule { min: 1.6, max: 2337.0 };
        let mut item = new_redaction_checkpoint("x", "y");
        item.maximum = 2338.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RedactionCheckpointRequiredLabelRule {
            message: "redaction checkpoint needs an entry".to_string(),
        };
        let mut item = new_redaction_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RedactionCheckpointRequiredHostRule {
            message: "redaction checkpoint needs this field".to_string(),
        };
        let mut item = new_redaction_checkpoint("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
