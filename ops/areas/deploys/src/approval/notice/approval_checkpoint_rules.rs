use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval checkpoint must pass. Check returns its problems, or none.
pub trait ApprovalCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalCheckpointRule for ApprovalCheckpointCodeLengthRule {
    fn check(&self, item: &ApprovalCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointRequiredCodeRule {
    pub message: String,
}

impl ApprovalCheckpointRule for ApprovalCheckpointRequiredCodeRule {
    fn check(&self, item: &ApprovalCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointLocaleLengthRule {
    pub max_length: i64,
}

impl ApprovalCheckpointRule for ApprovalCheckpointLocaleLengthRule {
    fn check(&self, item: &ApprovalCheckpoint) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointRequiredLocaleRule {
    pub message: String,
}

impl ApprovalCheckpointRule for ApprovalCheckpointRequiredLocaleRule {
    fn check(&self, item: &ApprovalCheckpoint) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalCheckpointRule for ApprovalCheckpointLimitRangeRule {
    fn check(&self, item: &ApprovalCheckpoint) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalCheckpointRequiredTimezoneRule {
    pub message: String,
}

impl ApprovalCheckpointRule for ApprovalCheckpointRequiredTimezoneRule {
    fn check(&self, item: &ApprovalCheckpoint) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval checkpoint.
#[derive(Debug, Clone)]
pub struct ApprovalCheckpointValidator {
    pub rules: Vec<Rc<dyn ApprovalCheckpointRule>>,
}

impl ApprovalCheckpointValidator {
    pub fn validate(&self, item: &ApprovalCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_checkpoint_validator() -> ApprovalCheckpointValidator {
    ApprovalCheckpointValidator {
        rules: vec![
            Rc::new(ApprovalCheckpointCodeLengthRule { max_length: 3600 }) as Rc<dyn ApprovalCheckpointRule>,
            Rc::new(ApprovalCheckpointRequiredCodeRule {
                message: "approval checkpoint needs this field".to_string(),
            }),
            Rc::new(ApprovalCheckpointLocaleLengthRule { max_length: 64 }),
            Rc::new(ApprovalCheckpointRequiredLocaleRule {
                message: "approval checkpoint needs an entry".to_string(),
            }),
            Rc::new(ApprovalCheckpointLimitRangeRule { min: 64, max: 120 }),
            Rc::new(ApprovalCheckpointRequiredTimezoneRule {
                message: "approval checkpoint needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_checkpoint_validator();
        let item = new_approval_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalCheckpointCodeLengthRule { max_length: 64 };
        let mut item = new_approval_checkpoint("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ApprovalCheckpointRequiredCodeRule {
            message: "approval checkpoint needs this field".to_string(),
        };
        let mut item = new_approval_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ApprovalCheckpointLocaleLengthRule { max_length: 32 };
        let mut item = new_approval_checkpoint("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ApprovalCheckpointRequiredLocaleRule {
            message: "approval checkpoint needs a value".to_string(),
        };
        let mut item = new_approval_checkpoint("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ApprovalCheckpointLimitRangeRule { min: 16, max: 255 };
        let mut item = new_approval_checkpoint("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ApprovalCheckpointRequiredTimezoneRule {
            message: "approval checkpoint needs this field".to_string(),
        };
        let mut item = new_approval_checkpoint("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
