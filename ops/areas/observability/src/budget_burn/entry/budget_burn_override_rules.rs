use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn override must pass. Check returns its problems, or none.
pub trait BudgetBurnOverrideRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnOverrideTitleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnOverrideRule for BudgetBurnOverrideTitleLengthRule {
    fn check(&self, item: &BudgetBurnOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnOverrideOwnerIdLengthRule {
    pub max_length: i64,
}

impl BudgetBurnOverrideRule for BudgetBurnOverrideOwnerIdLengthRule {
    fn check(&self, item: &BudgetBurnOverride) -> Vec<String> {
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
pub struct BudgetBurnOverrideQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnOverrideRule for BudgetBurnOverrideQuantityRangeRule {
    fn check(&self, item: &BudgetBurnOverride) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a budget burn override.
#[derive(Debug, Clone)]
pub struct BudgetBurnOverrideValidator {
    pub rules: Vec<Rc<dyn BudgetBurnOverrideRule>>,
}

impl BudgetBurnOverrideValidator {
    pub fn validate(&self, item: &BudgetBurnOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_override_validator() -> BudgetBurnOverrideValidator {
    BudgetBurnOverrideValidator {
        rules: vec![
            Rc::new(BudgetBurnOverrideTitleLengthRule { max_length: 3600 }) as Rc<dyn BudgetBurnOverrideRule>,
            Rc::new(BudgetBurnOverrideOwnerIdLengthRule { max_length: 120 }),
            Rc::new(BudgetBurnOverrideQuantityRangeRule { min: 80, max: 255 }),
        ],
    }
}
