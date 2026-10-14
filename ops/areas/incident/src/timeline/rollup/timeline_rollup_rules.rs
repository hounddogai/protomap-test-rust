use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline rollup must pass. Check returns its problems, or none.
pub trait TimelineRollupRule: std::fmt::Debug {
    fn check(&self, item: &TimelineRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRollupWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineRollupRule for TimelineRollupWeightRangeRule {
    fn check(&self, item: &TimelineRollup) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRollupRequiredCodeRule {
    pub message: String,
}

impl TimelineRollupRule for TimelineRollupRequiredCodeRule {
    fn check(&self, item: &TimelineRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRollupRequiredLabelRule {
    pub message: String,
}

impl TimelineRollupRule for TimelineRollupRequiredLabelRule {
    fn check(&self, item: &TimelineRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRollupPinnedGuardRule {
    pub limit: i64,
}

impl TimelineRollupRule for TimelineRollupPinnedGuardRule {
    fn check(&self, item: &TimelineRollup) -> Vec<String> {
        if item.pinned && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while pinned",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline rollup.
#[derive(Debug, Clone)]
pub struct TimelineRollupValidator {
    pub rules: Vec<Rc<dyn TimelineRollupRule>>,
}

impl TimelineRollupValidator {
    pub fn validate(&self, item: &TimelineRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_rollup_validator() -> TimelineRollupValidator {
    TimelineRollupValidator {
        rules: vec![
            Rc::new(TimelineRollupWeightRangeRule { min: 3.9, max: 3179.7 }) as Rc<dyn TimelineRollupRule>,
            Rc::new(TimelineRollupRequiredCodeRule {
                message: "timeline rollup needs an entry".to_string(),
            }),
            Rc::new(TimelineRollupRequiredLabelRule {
                message: "timeline rollup needs an entry".to_string(),
            }),
            Rc::new(TimelineRollupPinnedGuardRule { limit: 1000 }),
        ],
    }
}
