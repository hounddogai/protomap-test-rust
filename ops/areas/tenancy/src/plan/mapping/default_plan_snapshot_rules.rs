use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default plan snapshot must pass. Check returns its problems, or none.
pub trait DefaultPlanSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPlanSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanSnapshotRequiredNameRule {
    pub message: String,
}

impl DefaultPlanSnapshotRule for DefaultPlanSnapshotRequiredNameRule {
    fn check(&self, item: &DefaultPlanSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanSnapshotOwnerIdLengthRule {
    pub max_length: i64,
}

impl DefaultPlanSnapshotRule for DefaultPlanSnapshotOwnerIdLengthRule {
    fn check(&self, item: &DefaultPlanSnapshot) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanSnapshotVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultPlanSnapshotRule for DefaultPlanSnapshotVersionRangeRule {
    fn check(&self, item: &DefaultPlanSnapshot) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanSnapshotNameLengthRule {
    pub max_length: i64,
}

impl DefaultPlanSnapshotRule for DefaultPlanSnapshotNameLengthRule {
    fn check(&self, item: &DefaultPlanSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl DefaultPlanSnapshotRule for DefaultPlanSnapshotCodeLengthRule {
    fn check(&self, item: &DefaultPlanSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default plan snapshot.
#[derive(Debug, Clone)]
pub struct DefaultPlanSnapshotValidator {
    pub rules: Vec<Rc<dyn DefaultPlanSnapshotRule>>,
}

impl DefaultPlanSnapshotValidator {
    pub fn validate(&self, item: &DefaultPlanSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPlanSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPlanSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_plan_snapshot_validator() -> DefaultPlanSnapshotValidator {
    DefaultPlanSnapshotValidator {
        rules: vec![
            Rc::new(DefaultPlanSnapshotRequiredNameRule {
                message: "default plan snapshot needs an entry".to_string(),
            }) as Rc<dyn DefaultPlanSnapshotRule>,
            Rc::new(DefaultPlanSnapshotOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(DefaultPlanSnapshotVersionRangeRule { min: 16, max: 3600 }),
            Rc::new(DefaultPlanSnapshotNameLengthRule { max_length: 64 }),
            Rc::new(DefaultPlanSnapshotCodeLengthRule { max_length: 64 }),
        ],
    }
}
