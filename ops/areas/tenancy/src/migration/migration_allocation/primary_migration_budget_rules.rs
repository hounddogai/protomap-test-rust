use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary migration budget must pass. Check returns its problems, or none.
pub trait PrimaryMigrationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryMigrationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMigrationBudgetCodePrefixRule {
    pub prefix: String,
}

impl PrimaryMigrationBudgetRule for PrimaryMigrationBudgetCodePrefixRule {
    fn check(&self, item: &PrimaryMigrationBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMigrationBudgetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryMigrationBudgetRule for PrimaryMigrationBudgetMarginRangeRule {
    fn check(&self, item: &PrimaryMigrationBudget) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMigrationBudgetRequiredCodeRule {
    pub message: String,
}

impl PrimaryMigrationBudgetRule for PrimaryMigrationBudgetRequiredCodeRule {
    fn check(&self, item: &PrimaryMigrationBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMigrationBudgetPrincipalLengthRule {
    pub max_length: i64,
}

impl PrimaryMigrationBudgetRule for PrimaryMigrationBudgetPrincipalLengthRule {
    fn check(&self, item: &PrimaryMigrationBudget) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMigrationBudgetThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryMigrationBudgetRule for PrimaryMigrationBudgetThresholdRangeRule {
    fn check(&self, item: &PrimaryMigrationBudget) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryMigrationBudgetRequiredSegmentRule {
    pub message: String,
}

impl PrimaryMigrationBudgetRule for PrimaryMigrationBudgetRequiredSegmentRule {
    fn check(&self, item: &PrimaryMigrationBudget) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary migration budget.
#[derive(Debug, Clone)]
pub struct PrimaryMigrationBudgetValidator {
    pub rules: Vec<Rc<dyn PrimaryMigrationBudgetRule>>,
}

impl PrimaryMigrationBudgetValidator {
    pub fn validate(&self, item: &PrimaryMigrationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryMigrationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryMigrationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_migration_budget_validator() -> PrimaryMigrationBudgetValidator {
    PrimaryMigrationBudgetValidator {
        rules: vec![
            Rc::new(PrimaryMigrationBudgetCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PrimaryMigrationBudgetRule>,
            Rc::new(PrimaryMigrationBudgetMarginRangeRule { min: 1.5, max: 69.4 }),
            Rc::new(PrimaryMigrationBudgetRequiredCodeRule {
                message: "primary migration budget needs this field".to_string(),
            }),
            Rc::new(PrimaryMigrationBudgetPrincipalLengthRule { max_length: 32 }),
            Rc::new(PrimaryMigrationBudgetThresholdRangeRule { min: 8.1, max: 4746.2 }),
            Rc::new(PrimaryMigrationBudgetRequiredSegmentRule {
                message: "primary migration budget needs a value".to_string(),
            }),
        ],
    }
}
