use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled timeline checkpoint must pass. Check returns its problems, or none.
pub trait ScheduledTimelineCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTimelineCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineCheckpointActiveGuardRule {
    pub limit: i64,
}

impl ScheduledTimelineCheckpointRule for ScheduledTimelineCheckpointActiveGuardRule {
    fn check(&self, item: &ScheduledTimelineCheckpoint) -> Vec<String> {
        if item.active && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while active",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineCheckpointRequiredCodeRule {
    pub message: String,
}

impl ScheduledTimelineCheckpointRule for ScheduledTimelineCheckpointRequiredCodeRule {
    fn check(&self, item: &ScheduledTimelineCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineCheckpointRequiredChannelRule {
    pub message: String,
}

impl ScheduledTimelineCheckpointRule for ScheduledTimelineCheckpointRequiredChannelRule {
    fn check(&self, item: &ScheduledTimelineCheckpoint) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled timeline checkpoint.
#[derive(Debug, Clone)]
pub struct ScheduledTimelineCheckpointValidator {
    pub rules: Vec<Rc<dyn ScheduledTimelineCheckpointRule>>,
}

impl ScheduledTimelineCheckpointValidator {
    pub fn validate(&self, item: &ScheduledTimelineCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledTimelineCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTimelineCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_timeline_checkpoint_validator() -> ScheduledTimelineCheckpointValidator {
    ScheduledTimelineCheckpointValidator {
        rules: vec![
            Rc::new(ScheduledTimelineCheckpointActiveGuardRule { limit: 1000 })
                as Rc<dyn ScheduledTimelineCheckpointRule>,
            Rc::new(ScheduledTimelineCheckpointRequiredCodeRule {
                message: "scheduled timeline checkpoint needs a value".to_string(),
            }),
            Rc::new(ScheduledTimelineCheckpointRequiredChannelRule {
                message: "scheduled timeline checkpoint needs this field".to_string(),
            }),
        ],
    }
}
