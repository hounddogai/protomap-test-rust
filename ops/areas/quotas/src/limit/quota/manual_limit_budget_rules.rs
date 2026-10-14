use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual limit budget must pass. Check returns its problems, or none.
pub trait ManualLimitBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ManualLimitBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitBudgetCountryLengthRule {
    pub max_length: i64,
}

impl ManualLimitBudgetRule for ManualLimitBudgetCountryLengthRule {
    fn check(&self, item: &ManualLimitBudget) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitBudgetOwnerIdLengthRule {
    pub max_length: i64,
}

impl ManualLimitBudgetRule for ManualLimitBudgetOwnerIdLengthRule {
    fn check(&self, item: &ManualLimitBudget) -> Vec<String> {
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
pub struct ManualLimitBudgetRequiredOwnerIdRule {
    pub message: String,
}

impl ManualLimitBudgetRule for ManualLimitBudgetRequiredOwnerIdRule {
    fn check(&self, item: &ManualLimitBudget) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual limit budget.
#[derive(Debug, Clone)]
pub struct ManualLimitBudgetValidator {
    pub rules: Vec<Rc<dyn ManualLimitBudgetRule>>,
}

impl ManualLimitBudgetValidator {
    pub fn validate(&self, item: &ManualLimitBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualLimitBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualLimitBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_limit_budget_validator() -> ManualLimitBudgetValidator {
    ManualLimitBudgetValidator {
        rules: vec![
            Rc::new(ManualLimitBudgetCountryLengthRule { max_length: 86400 }) as Rc<dyn ManualLimitBudgetRule>,
            Rc::new(ManualLimitBudgetOwnerIdLengthRule { max_length: 255 }),
            Rc::new(ManualLimitBudgetRequiredOwnerIdRule {
                message: "manual limit budget needs this field".to_string(),
            }),
        ],
    }
}
