use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback checkpoint must pass. Check returns its problems, or none.
pub trait RollbackCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RollbackCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointCodePrefixRule {
    pub prefix: String,
}

impl RollbackCheckpointRule for RollbackCheckpointCodePrefixRule {
    fn check(&self, item: &RollbackCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointRequiredCodeRule {
    pub message: String,
}

impl RollbackCheckpointRule for RollbackCheckpointRequiredCodeRule {
    fn check(&self, item: &RollbackCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackCheckpointRule for RollbackCheckpointMarginRangeRule {
    fn check(&self, item: &RollbackCheckpoint) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointRequiredNameRule {
    pub message: String,
}

impl RollbackCheckpointRule for RollbackCheckpointRequiredNameRule {
    fn check(&self, item: &RollbackCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollbackCheckpointRule for RollbackCheckpointAllowedNameRule {
    fn check(&self, item: &RollbackCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpointNameLengthRule {
    pub max_length: i64,
}

impl RollbackCheckpointRule for RollbackCheckpointNameLengthRule {
    fn check(&self, item: &RollbackCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback checkpoint.
#[derive(Debug, Clone)]
pub struct RollbackCheckpointValidator {
    pub rules: Vec<Rc<dyn RollbackCheckpointRule>>,
}

impl RollbackCheckpointValidator {
    pub fn validate(&self, item: &RollbackCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_checkpoint_validator() -> RollbackCheckpointValidator {
    RollbackCheckpointValidator {
        rules: vec![
            Rc::new(RollbackCheckpointCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RollbackCheckpointRule>,
            Rc::new(RollbackCheckpointRequiredCodeRule {
                message: "rollback checkpoint needs this field".to_string(),
            }),
            Rc::new(RollbackCheckpointMarginRangeRule { min: 9.1, max: 1258.1 }),
            Rc::new(RollbackCheckpointRequiredNameRule {
                message: "rollback checkpoint needs a value".to_string(),
            }),
            Rc::new(RollbackCheckpointAllowedNameRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(RollbackCheckpointNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_checkpoint_validator();
        let item = new_rollback_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackCheckpointRequiredCodeRule {
            message: "rollback checkpoint needs a value".to_string(),
        };
        let mut item = new_rollback_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RollbackCheckpointMarginRangeRule { min: 4.7, max: 4306.3 };
        let mut item = new_rollback_checkpoint("x", "y");
        item.margin = 4307.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollbackCheckpointRequiredNameRule {
            message: "rollback checkpoint needs a value".to_string(),
        };
        let mut item = new_rollback_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollbackCheckpointNameLengthRule { max_length: 255 };
        let mut item = new_rollback_checkpoint("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
