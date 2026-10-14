use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn threshold must pass. Check returns its problems, or none.
pub trait BudgetBurnThresholdRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThresholdTitleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnThresholdRule for BudgetBurnThresholdTitleLengthRule {
    fn check(&self, item: &BudgetBurnThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThresholdRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnThresholdRule for BudgetBurnThresholdRequiredCodeRule {
    fn check(&self, item: &BudgetBurnThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThresholdQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnThresholdRule for BudgetBurnThresholdQuotaUnitsRangeRule {
    fn check(&self, item: &BudgetBurnThreshold) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThresholdRegionLengthRule {
    pub max_length: i64,
}

impl BudgetBurnThresholdRule for BudgetBurnThresholdRegionLengthRule {
    fn check(&self, item: &BudgetBurnThreshold) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThresholdRequiredTitleRule {
    pub message: String,
}

impl BudgetBurnThresholdRule for BudgetBurnThresholdRequiredTitleRule {
    fn check(&self, item: &BudgetBurnThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThresholdAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnThresholdRule for BudgetBurnThresholdAllowedTitleRule {
    fn check(&self, item: &BudgetBurnThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn threshold.
#[derive(Debug, Clone)]
pub struct BudgetBurnThresholdValidator {
    pub rules: Vec<Rc<dyn BudgetBurnThresholdRule>>,
}

impl BudgetBurnThresholdValidator {
    pub fn validate(&self, item: &BudgetBurnThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_threshold_validator() -> BudgetBurnThresholdValidator {
    BudgetBurnThresholdValidator {
        rules: vec![
            Rc::new(BudgetBurnThresholdTitleLengthRule { max_length: 3600 }) as Rc<dyn BudgetBurnThresholdRule>,
            Rc::new(BudgetBurnThresholdRequiredCodeRule {
                message: "budget burn threshold needs this field".to_string(),
            }),
            Rc::new(BudgetBurnThresholdQuotaUnitsRangeRule { min: 32, max: 3600 }),
            Rc::new(BudgetBurnThresholdRegionLengthRule { max_length: 64 }),
            Rc::new(BudgetBurnThresholdRequiredTitleRule {
                message: "budget burn threshold needs this field".to_string(),
            }),
            Rc::new(BudgetBurnThresholdAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}
