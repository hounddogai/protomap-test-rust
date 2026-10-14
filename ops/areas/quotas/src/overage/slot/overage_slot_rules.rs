use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage slot must pass. Check returns its problems, or none.
pub trait OverageSlotRule: std::fmt::Debug {
    fn check(&self, item: &OverageSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSlotTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl OverageSlotRule for OverageSlotTagsLimitRule {
    fn check(&self, item: &OverageSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSlotTitleLengthRule {
    pub max_length: i64,
}

impl OverageSlotRule for OverageSlotTitleLengthRule {
    fn check(&self, item: &OverageSlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSlotRequiredTitleRule {
    pub message: String,
}

impl OverageSlotRule for OverageSlotRequiredTitleRule {
    fn check(&self, item: &OverageSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSlotReviewerLengthRule {
    pub max_length: i64,
}

impl OverageSlotRule for OverageSlotReviewerLengthRule {
    fn check(&self, item: &OverageSlot) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage slot.
#[derive(Debug, Clone)]
pub struct OverageSlotValidator {
    pub rules: Vec<Rc<dyn OverageSlotRule>>,
}

impl OverageSlotValidator {
    pub fn validate(&self, item: &OverageSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_slot_validator() -> OverageSlotValidator {
    OverageSlotValidator {
        rules: vec![
            Rc::new(OverageSlotTagsLimitRule {
                max_count: 1000,
                banned: vec!["strict".to_string(), "capped".to_string(), "batched".to_string()],
            }) as Rc<dyn OverageSlotRule>,
            Rc::new(OverageSlotTitleLengthRule { max_length: 255 }),
            Rc::new(OverageSlotRequiredTitleRule {
                message: "overage slot needs this field".to_string(),
            }),
            Rc::new(OverageSlotReviewerLengthRule { max_length: 16 }),
        ],
    }
}
