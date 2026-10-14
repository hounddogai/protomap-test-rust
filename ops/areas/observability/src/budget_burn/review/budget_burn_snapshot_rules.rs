use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn snapshot must pass. Check returns its problems, or none.
pub trait BudgetBurnSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSnapshotTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnSnapshotRule for BudgetBurnSnapshotTtlSecondsRangeRule {
    fn check(&self, item: &BudgetBurnSnapshot) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnSnapshotRule for BudgetBurnSnapshotCodeLengthRule {
    fn check(&self, item: &BudgetBurnSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSnapshotRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnSnapshotRule for BudgetBurnSnapshotRequiredCodeRule {
    fn check(&self, item: &BudgetBurnSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSnapshotBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnSnapshotRule for BudgetBurnSnapshotBurstRangeRule {
    fn check(&self, item: &BudgetBurnSnapshot) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSnapshotAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnSnapshotRule for BudgetBurnSnapshotAllowedLabelRule {
    fn check(&self, item: &BudgetBurnSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn snapshot.
#[derive(Debug, Clone)]
pub struct BudgetBurnSnapshotValidator {
    pub rules: Vec<Rc<dyn BudgetBurnSnapshotRule>>,
}

impl BudgetBurnSnapshotValidator {
    pub fn validate(&self, item: &BudgetBurnSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_snapshot_validator() -> BudgetBurnSnapshotValidator {
    BudgetBurnSnapshotValidator {
        rules: vec![
            Rc::new(BudgetBurnSnapshotTtlSecondsRangeRule { min: 32, max: 120 }) as Rc<dyn BudgetBurnSnapshotRule>,
            Rc::new(BudgetBurnSnapshotCodeLengthRule { max_length: 32 }),
            Rc::new(BudgetBurnSnapshotRequiredCodeRule {
                message: "budget burn snapshot needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnSnapshotBurstRangeRule { min: 32, max: 255 }),
            Rc::new(BudgetBurnSnapshotAllowedLabelRule { allowed: Vec::new() }),
        ],
    }
}
