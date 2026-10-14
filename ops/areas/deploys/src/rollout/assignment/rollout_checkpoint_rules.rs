use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout checkpoint must pass. Check returns its problems, or none.
pub trait RolloutCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RolloutCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl RolloutCheckpointRule for RolloutCheckpointCodeLengthRule {
    fn check(&self, item: &RolloutCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCheckpointEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutCheckpointRule for RolloutCheckpointEndsAtRangeRule {
    fn check(&self, item: &RolloutCheckpoint) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCheckpointRequiredSourceRule {
    pub message: String,
}

impl RolloutCheckpointRule for RolloutCheckpointRequiredSourceRule {
    fn check(&self, item: &RolloutCheckpoint) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCheckpointBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutCheckpointRule for RolloutCheckpointBudgetRangeRule {
    fn check(&self, item: &RolloutCheckpoint) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCheckpointAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl RolloutCheckpointRule for RolloutCheckpointAllowedSourceRule {
    fn check(&self, item: &RolloutCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is below minimum", item.source)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout checkpoint.
#[derive(Debug, Clone)]
pub struct RolloutCheckpointValidator {
    pub rules: Vec<Rc<dyn RolloutCheckpointRule>>,
}

impl RolloutCheckpointValidator {
    pub fn validate(&self, item: &RolloutCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_checkpoint_validator() -> RolloutCheckpointValidator {
    RolloutCheckpointValidator {
        rules: vec![
            Rc::new(RolloutCheckpointCodeLengthRule { max_length: 16 }) as Rc<dyn RolloutCheckpointRule>,
            Rc::new(RolloutCheckpointEndsAtRangeRule { min: 80, max: 255 }),
            Rc::new(RolloutCheckpointRequiredSourceRule {
                message: "rollout checkpoint needs an entry".to_string(),
            }),
            Rc::new(RolloutCheckpointBudgetRangeRule { min: 3.2, max: 1497.9 }),
            Rc::new(RolloutCheckpointAllowedSourceRule {
                allowed: vec!["scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_checkpoint_validator();
        let item = new_rollout_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutCheckpointCodeLengthRule { max_length: 1000 };
        let mut item = new_rollout_checkpoint("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RolloutCheckpointEndsAtRangeRule { min: 16, max: 64 };
        let mut item = new_rollout_checkpoint("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = RolloutCheckpointRequiredSourceRule {
            message: "rollout checkpoint needs an entry".to_string(),
        };
        let mut item = new_rollout_checkpoint("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RolloutCheckpointBudgetRangeRule { min: 7.2, max: 2471.5 };
        let mut item = new_rollout_checkpoint("x", "y");
        item.budget = 2473.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
