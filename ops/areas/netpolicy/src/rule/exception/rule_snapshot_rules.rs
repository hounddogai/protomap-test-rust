use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule snapshot must pass. Check returns its problems, or none.
pub trait RuleSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RuleSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSnapshotCodePrefixRule {
    pub prefix: String,
}

impl RuleSnapshotRule for RuleSnapshotCodePrefixRule {
    fn check(&self, item: &RuleSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl RuleSnapshotRule for RuleSnapshotCodeLengthRule {
    fn check(&self, item: &RuleSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSnapshotCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleSnapshotRule for RuleSnapshotCapacityRangeRule {
    fn check(&self, item: &RuleSnapshot) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule snapshot.
#[derive(Debug, Clone)]
pub struct RuleSnapshotValidator {
    pub rules: Vec<Rc<dyn RuleSnapshotRule>>,
}

impl RuleSnapshotValidator {
    pub fn validate(&self, item: &RuleSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_snapshot_validator() -> RuleSnapshotValidator {
    RuleSnapshotValidator {
        rules: vec![
            Rc::new(RuleSnapshotCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RuleSnapshotRule>,
            Rc::new(RuleSnapshotCodeLengthRule { max_length: 32 }),
            Rc::new(RuleSnapshotCapacityRangeRule { min: 32, max: 64 }),
        ],
    }
}
