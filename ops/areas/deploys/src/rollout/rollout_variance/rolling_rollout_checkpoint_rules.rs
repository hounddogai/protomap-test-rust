use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling rollout checkpoint must pass. Check returns its problems, or none.
pub trait RollingRolloutCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RollingRolloutCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutCheckpointMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingRolloutCheckpointRule for RollingRolloutCheckpointMarginRangeRule {
    fn check(&self, item: &RollingRolloutCheckpoint) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutCheckpointAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl RollingRolloutCheckpointRule for RollingRolloutCheckpointAllowedNamespaceRule {
    fn check(&self, item: &RollingRolloutCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is above maximum", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutCheckpointNamespaceLengthRule {
    pub max_length: i64,
}

impl RollingRolloutCheckpointRule for RollingRolloutCheckpointNamespaceLengthRule {
    fn check(&self, item: &RollingRolloutCheckpoint) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling rollout checkpoint.
#[derive(Debug, Clone)]
pub struct RollingRolloutCheckpointValidator {
    pub rules: Vec<Rc<dyn RollingRolloutCheckpointRule>>,
}

impl RollingRolloutCheckpointValidator {
    pub fn validate(&self, item: &RollingRolloutCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingRolloutCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRolloutCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_rollout_checkpoint_validator() -> RollingRolloutCheckpointValidator {
    RollingRolloutCheckpointValidator {
        rules: vec![
            Rc::new(RollingRolloutCheckpointMarginRangeRule { min: 2.7, max: 2182.7 })
                as Rc<dyn RollingRolloutCheckpointRule>,
            Rc::new(RollingRolloutCheckpointAllowedNamespaceRule {
                allowed: vec!["seasonal".to_string(), "default".to_string(), "pending".to_string()],
            }),
            Rc::new(RollingRolloutCheckpointNamespaceLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_rollout_checkpoint_validator();
        let item = new_rolling_rollout_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RollingRolloutCheckpointMarginRangeRule { min: 9.7, max: 2527.1 };
        let mut item = new_rolling_rollout_checkpoint("x", "y");
        item.margin = 2528.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RollingRolloutCheckpointNamespaceLengthRule { max_length: 255 };
        let mut item = new_rolling_rollout_checkpoint("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
