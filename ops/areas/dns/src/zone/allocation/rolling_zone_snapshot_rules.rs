use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling zone snapshot must pass. Check returns its problems, or none.
pub trait RollingZoneSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RollingZoneSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSnapshotRequiredNameRule {
    pub message: String,
}

impl RollingZoneSnapshotRule for RollingZoneSnapshotRequiredNameRule {
    fn check(&self, item: &RollingZoneSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSnapshotRequiredCodeRule {
    pub message: String,
}

impl RollingZoneSnapshotRule for RollingZoneSnapshotRequiredCodeRule {
    fn check(&self, item: &RollingZoneSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSnapshotRequiredScopeRule {
    pub message: String,
}

impl RollingZoneSnapshotRule for RollingZoneSnapshotRequiredScopeRule {
    fn check(&self, item: &RollingZoneSnapshot) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSnapshotCodePrefixRule {
    pub prefix: String,
}

impl RollingZoneSnapshotRule for RollingZoneSnapshotCodePrefixRule {
    fn check(&self, item: &RollingZoneSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl RollingZoneSnapshotRule for RollingZoneSnapshotCodeLengthRule {
    fn check(&self, item: &RollingZoneSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling zone snapshot.
#[derive(Debug, Clone)]
pub struct RollingZoneSnapshotValidator {
    pub rules: Vec<Rc<dyn RollingZoneSnapshotRule>>,
}

impl RollingZoneSnapshotValidator {
    pub fn validate(&self, item: &RollingZoneSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingZoneSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingZoneSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_zone_snapshot_validator() -> RollingZoneSnapshotValidator {
    RollingZoneSnapshotValidator {
        rules: vec![
            Rc::new(RollingZoneSnapshotRequiredNameRule {
                message: "rolling zone snapshot needs this field".to_string(),
            }) as Rc<dyn RollingZoneSnapshotRule>,
            Rc::new(RollingZoneSnapshotRequiredCodeRule {
                message: "rolling zone snapshot needs an entry".to_string(),
            }),
            Rc::new(RollingZoneSnapshotRequiredScopeRule {
                message: "rolling zone snapshot needs an entry".to_string(),
            }),
            Rc::new(RollingZoneSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RollingZoneSnapshotCodeLengthRule { max_length: 64 }),
        ],
    }
}
