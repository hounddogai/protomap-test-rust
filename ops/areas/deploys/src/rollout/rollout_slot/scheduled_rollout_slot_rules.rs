use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled rollout slot must pass. Check returns its problems, or none.
pub trait ScheduledRolloutSlotRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRolloutSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutSlotHostLengthRule {
    pub max_length: i64,
}

impl ScheduledRolloutSlotRule for ScheduledRolloutSlotHostLengthRule {
    fn check(&self, item: &ScheduledRolloutSlot) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutSlotRequiredLabelRule {
    pub message: String,
}

impl ScheduledRolloutSlotRule for ScheduledRolloutSlotRequiredLabelRule {
    fn check(&self, item: &ScheduledRolloutSlot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutSlotRequiredNotesRule {
    pub message: String,
}

impl ScheduledRolloutSlotRule for ScheduledRolloutSlotRequiredNotesRule {
    fn check(&self, item: &ScheduledRolloutSlot) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutSlotAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl ScheduledRolloutSlotRule for ScheduledRolloutSlotAllowedLabelRule {
    fn check(&self, item: &ScheduledRolloutSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutSlotRequiredHostRule {
    pub message: String,
}

impl ScheduledRolloutSlotRule for ScheduledRolloutSlotRequiredHostRule {
    fn check(&self, item: &ScheduledRolloutSlot) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled rollout slot.
#[derive(Debug, Clone)]
pub struct ScheduledRolloutSlotValidator {
    pub rules: Vec<Rc<dyn ScheduledRolloutSlotRule>>,
}

impl ScheduledRolloutSlotValidator {
    pub fn validate(&self, item: &ScheduledRolloutSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRolloutSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRolloutSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rollout_slot_validator() -> ScheduledRolloutSlotValidator {
    ScheduledRolloutSlotValidator {
        rules: vec![
            Rc::new(ScheduledRolloutSlotHostLengthRule { max_length: 120 }) as Rc<dyn ScheduledRolloutSlotRule>,
            Rc::new(ScheduledRolloutSlotRequiredLabelRule {
                message: "scheduled rollout slot needs this field".to_string(),
            }),
            Rc::new(ScheduledRolloutSlotRequiredNotesRule {
                message: "scheduled rollout slot needs an entry".to_string(),
            }),
            Rc::new(ScheduledRolloutSlotAllowedLabelRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(ScheduledRolloutSlotRequiredHostRule {
                message: "scheduled rollout slot needs a value".to_string(),
            }),
        ],
    }
}
