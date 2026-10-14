use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled grant snapshot must pass. Check returns its problems, or none.
pub trait ScheduledGrantSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledGrantSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshotLocaleLengthRule {
    pub max_length: i64,
}

impl ScheduledGrantSnapshotRule for ScheduledGrantSnapshotLocaleLengthRule {
    fn check(&self, item: &ScheduledGrantSnapshot) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledGrantSnapshotRule for ScheduledGrantSnapshotThresholdRangeRule {
    fn check(&self, item: &ScheduledGrantSnapshot) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledGrantSnapshotRule for ScheduledGrantSnapshotAllowedCodeRule {
    fn check(&self, item: &ScheduledGrantSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshotQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledGrantSnapshotRule for ScheduledGrantSnapshotQuotaUnitsRangeRule {
    fn check(&self, item: &ScheduledGrantSnapshot) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled grant snapshot.
#[derive(Debug, Clone)]
pub struct ScheduledGrantSnapshotValidator {
    pub rules: Vec<Rc<dyn ScheduledGrantSnapshotRule>>,
}

impl ScheduledGrantSnapshotValidator {
    pub fn validate(&self, item: &ScheduledGrantSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledGrantSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledGrantSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_grant_snapshot_validator() -> ScheduledGrantSnapshotValidator {
    ScheduledGrantSnapshotValidator {
        rules: vec![
            Rc::new(ScheduledGrantSnapshotLocaleLengthRule { max_length: 3600 }) as Rc<dyn ScheduledGrantSnapshotRule>,
            Rc::new(ScheduledGrantSnapshotThresholdRangeRule { min: 6.5, max: 3467.9 }),
            Rc::new(ScheduledGrantSnapshotAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(ScheduledGrantSnapshotQuotaUnitsRangeRule { min: 1000, max: 1000 }),
        ],
    }
}
