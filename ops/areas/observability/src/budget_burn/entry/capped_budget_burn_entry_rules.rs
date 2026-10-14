use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped budget burn entry must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnEntryRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnEntryRequiredReviewerRule {
    pub message: String,
}

impl CappedBudgetBurnEntryRule for CappedBudgetBurnEntryRequiredReviewerRule {
    fn check(&self, item: &CappedBudgetBurnEntry) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnEntryRequiredNameRule {
    pub message: String,
}

impl CappedBudgetBurnEntryRule for CappedBudgetBurnEntryRequiredNameRule {
    fn check(&self, item: &CappedBudgetBurnEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnEntryAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CappedBudgetBurnEntryRule for CappedBudgetBurnEntryAllowedNameRule {
    fn check(&self, item: &CappedBudgetBurnEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped budget burn entry.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnEntryValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnEntryRule>>,
}

impl CappedBudgetBurnEntryValidator {
    pub fn validate(&self, item: &CappedBudgetBurnEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedBudgetBurnEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_entry_validator() -> CappedBudgetBurnEntryValidator {
    CappedBudgetBurnEntryValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnEntryRequiredReviewerRule {
                message: "capped budget burn entry needs a value".to_string(),
            }) as Rc<dyn CappedBudgetBurnEntryRule>,
            Rc::new(CappedBudgetBurnEntryRequiredNameRule {
                message: "capped budget burn entry needs a value".to_string(),
            }),
            Rc::new(CappedBudgetBurnEntryAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}
