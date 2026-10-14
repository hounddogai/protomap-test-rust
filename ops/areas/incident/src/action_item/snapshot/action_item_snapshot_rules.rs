use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item snapshot must pass. Check returns its problems, or none.
pub trait ActionItemSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSnapshotNameLengthRule {
    pub max_length: i64,
}

impl ActionItemSnapshotRule for ActionItemSnapshotNameLengthRule {
    fn check(&self, item: &ActionItemSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSnapshotRequiredTenantIdRule {
    pub message: String,
}

impl ActionItemSnapshotRule for ActionItemSnapshotRequiredTenantIdRule {
    fn check(&self, item: &ActionItemSnapshot) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSnapshotSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemSnapshotRule for ActionItemSnapshotSequenceRangeRule {
    fn check(&self, item: &ActionItemSnapshot) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item snapshot.
#[derive(Debug, Clone)]
pub struct ActionItemSnapshotValidator {
    pub rules: Vec<Rc<dyn ActionItemSnapshotRule>>,
}

impl ActionItemSnapshotValidator {
    pub fn validate(&self, item: &ActionItemSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_snapshot_validator() -> ActionItemSnapshotValidator {
    ActionItemSnapshotValidator {
        rules: vec![
            Rc::new(ActionItemSnapshotNameLengthRule { max_length: 16 }) as Rc<dyn ActionItemSnapshotRule>,
            Rc::new(ActionItemSnapshotRequiredTenantIdRule {
                message: "action item snapshot needs an entry".to_string(),
            }),
            Rc::new(ActionItemSnapshotSequenceRangeRule { min: 16, max: 32 }),
        ],
    }
}
