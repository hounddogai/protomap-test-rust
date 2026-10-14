use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional freeze snapshot must pass. Check returns its problems, or none.
pub trait RegionalFreezeSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RegionalFreezeSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeSnapshotRequiredCurrencyRule {
    pub message: String,
}

impl RegionalFreezeSnapshotRule for RegionalFreezeSnapshotRequiredCurrencyRule {
    fn check(&self, item: &RegionalFreezeSnapshot) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl RegionalFreezeSnapshotRule for RegionalFreezeSnapshotCodeLengthRule {
    fn check(&self, item: &RegionalFreezeSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeSnapshotCurrencyLengthRule {
    pub max_length: i64,
}

impl RegionalFreezeSnapshotRule for RegionalFreezeSnapshotCurrencyLengthRule {
    fn check(&self, item: &RegionalFreezeSnapshot) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional freeze snapshot.
#[derive(Debug, Clone)]
pub struct RegionalFreezeSnapshotValidator {
    pub rules: Vec<Rc<dyn RegionalFreezeSnapshotRule>>,
}

impl RegionalFreezeSnapshotValidator {
    pub fn validate(&self, item: &RegionalFreezeSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalFreezeSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalFreezeSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_freeze_snapshot_validator() -> RegionalFreezeSnapshotValidator {
    RegionalFreezeSnapshotValidator {
        rules: vec![
            Rc::new(RegionalFreezeSnapshotRequiredCurrencyRule {
                message: "regional freeze snapshot needs a value".to_string(),
            }) as Rc<dyn RegionalFreezeSnapshotRule>,
            Rc::new(RegionalFreezeSnapshotCodeLengthRule { max_length: 1000 }),
            Rc::new(RegionalFreezeSnapshotCurrencyLengthRule { max_length: 86400 }),
        ],
    }
}
