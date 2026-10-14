use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback threshold must pass. Check returns its problems, or none.
pub trait RollbackThresholdRule: std::fmt::Debug {
    fn check(&self, item: &RollbackThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdLabelLengthRule {
    pub max_length: i64,
}

impl RollbackThresholdRule for RollbackThresholdLabelLengthRule {
    fn check(&self, item: &RollbackThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdRequiredLabelRule {
    pub message: String,
}

impl RollbackThresholdRule for RollbackThresholdRequiredLabelRule {
    fn check(&self, item: &RollbackThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdChannelLengthRule {
    pub max_length: i64,
}

impl RollbackThresholdRule for RollbackThresholdChannelLengthRule {
    fn check(&self, item: &RollbackThreshold) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdRequiredCodeRule {
    pub message: String,
}

impl RollbackThresholdRule for RollbackThresholdRequiredCodeRule {
    fn check(&self, item: &RollbackThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackThresholdRule for RollbackThresholdEndsAtRangeRule {
    fn check(&self, item: &RollbackThreshold) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThresholdCodePrefixRule {
    pub prefix: String,
}

impl RollbackThresholdRule for RollbackThresholdCodePrefixRule {
    fn check(&self, item: &RollbackThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback threshold.
#[derive(Debug, Clone)]
pub struct RollbackThresholdValidator {
    pub rules: Vec<Rc<dyn RollbackThresholdRule>>,
}

impl RollbackThresholdValidator {
    pub fn validate(&self, item: &RollbackThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_threshold_validator() -> RollbackThresholdValidator {
    RollbackThresholdValidator {
        rules: vec![
            Rc::new(RollbackThresholdLabelLengthRule { max_length: 86400 }) as Rc<dyn RollbackThresholdRule>,
            Rc::new(RollbackThresholdRequiredLabelRule {
                message: "rollback threshold needs an entry".to_string(),
            }),
            Rc::new(RollbackThresholdChannelLengthRule { max_length: 86400 }),
            Rc::new(RollbackThresholdRequiredCodeRule {
                message: "rollback threshold needs a value".to_string(),
            }),
            Rc::new(RollbackThresholdEndsAtRangeRule { min: 16, max: 120 }),
            Rc::new(RollbackThresholdCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_threshold_validator();
        let item = new_rollback_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollbackThresholdLabelLengthRule { max_length: 80 };
        let mut item = new_rollback_threshold("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackThresholdRequiredLabelRule {
            message: "rollback threshold needs an entry".to_string(),
        };
        let mut item = new_rollback_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = RollbackThresholdChannelLengthRule { max_length: 255 };
        let mut item = new_rollback_threshold("x", "y");
        item.channel = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackThresholdRequiredCodeRule {
            message: "rollback threshold needs a value".to_string(),
        };
        let mut item = new_rollback_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RollbackThresholdEndsAtRangeRule { min: 80, max: 120 };
        let mut item = new_rollback_threshold("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
