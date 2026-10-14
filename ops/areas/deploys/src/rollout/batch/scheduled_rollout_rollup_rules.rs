use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled rollout rollup must pass. Check returns its problems, or none.
pub trait ScheduledRolloutRollupRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRolloutRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutRollupAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRolloutRollupRule for ScheduledRolloutRollupAttemptsRangeRule {
    fn check(&self, item: &ScheduledRolloutRollup) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutRollupCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRolloutRollupRule for ScheduledRolloutRollupCodePrefixRule {
    fn check(&self, item: &ScheduledRolloutRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutRollupPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRolloutRollupRule for ScheduledRolloutRollupPriorityRangeRule {
    fn check(&self, item: &ScheduledRolloutRollup) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled rollout rollup.
#[derive(Debug, Clone)]
pub struct ScheduledRolloutRollupValidator {
    pub rules: Vec<Rc<dyn ScheduledRolloutRollupRule>>,
}

impl ScheduledRolloutRollupValidator {
    pub fn validate(&self, item: &ScheduledRolloutRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledRolloutRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRolloutRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rollout_rollup_validator() -> ScheduledRolloutRollupValidator {
    ScheduledRolloutRollupValidator {
        rules: vec![
            Rc::new(ScheduledRolloutRollupAttemptsRangeRule { min: 255, max: 3600 })
                as Rc<dyn ScheduledRolloutRollupRule>,
            Rc::new(ScheduledRolloutRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ScheduledRolloutRollupPriorityRangeRule { min: 80, max: 255 }),
        ],
    }
}
