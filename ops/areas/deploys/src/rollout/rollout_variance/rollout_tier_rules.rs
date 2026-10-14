use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout tier must pass. Check returns its problems, or none.
pub trait RolloutTierRule: std::fmt::Debug {
    fn check(&self, item: &RolloutTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTierBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutTierRule for RolloutTierBatchSizeRangeRule {
    fn check(&self, item: &RolloutTier) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTierCodePrefixRule {
    pub prefix: String,
}

impl RolloutTierRule for RolloutTierCodePrefixRule {
    fn check(&self, item: &RolloutTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutTierRule for RolloutTierRatioRangeRule {
    fn check(&self, item: &RolloutTier) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTierRequiredTitleRule {
    pub message: String,
}

impl RolloutTierRule for RolloutTierRequiredTitleRule {
    fn check(&self, item: &RolloutTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout tier.
#[derive(Debug, Clone)]
pub struct RolloutTierValidator {
    pub rules: Vec<Rc<dyn RolloutTierRule>>,
}

impl RolloutTierValidator {
    pub fn validate(&self, item: &RolloutTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_tier_validator() -> RolloutTierValidator {
    RolloutTierValidator {
        rules: vec![
            Rc::new(RolloutTierBatchSizeRangeRule { min: 64, max: 255 }) as Rc<dyn RolloutTierRule>,
            Rc::new(RolloutTierCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RolloutTierRatioRangeRule { min: 9.5, max: 3711.4 }),
            Rc::new(RolloutTierRequiredTitleRule {
                message: "rollout tier needs an entry".to_string(),
            }),
        ],
    }
}
