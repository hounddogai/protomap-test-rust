use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin snapshot must pass. Check returns its problems, or none.
pub trait RegionPinSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSnapshotAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl RegionPinSnapshotRule for RegionPinSnapshotAllowedUnitRule {
    fn check(&self, item: &RegionPinSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is missing", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSnapshotRequiredCodeRule {
    pub message: String,
}

impl RegionPinSnapshotRule for RegionPinSnapshotRequiredCodeRule {
    fn check(&self, item: &RegionPinSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSnapshotNameLengthRule {
    pub max_length: i64,
}

impl RegionPinSnapshotRule for RegionPinSnapshotNameLengthRule {
    fn check(&self, item: &RegionPinSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin snapshot.
#[derive(Debug, Clone)]
pub struct RegionPinSnapshotValidator {
    pub rules: Vec<Rc<dyn RegionPinSnapshotRule>>,
}

impl RegionPinSnapshotValidator {
    pub fn validate(&self, item: &RegionPinSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_snapshot_validator() -> RegionPinSnapshotValidator {
    RegionPinSnapshotValidator {
        rules: vec![
            Rc::new(RegionPinSnapshotAllowedUnitRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn RegionPinSnapshotRule>,
            Rc::new(RegionPinSnapshotRequiredCodeRule {
                message: "region pin snapshot needs this field".to_string(),
            }),
            Rc::new(RegionPinSnapshotNameLengthRule { max_length: 86400 }),
        ],
    }
}
