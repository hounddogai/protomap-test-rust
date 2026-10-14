use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending verifier budget must pass. Check returns its problems, or none.
pub trait PendingVerifierBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PendingVerifierBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierBudgetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PendingVerifierBudgetRule for PendingVerifierBudgetAllowedNameRule {
    fn check(&self, item: &PendingVerifierBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierBudgetNameLengthRule {
    pub max_length: i64,
}

impl PendingVerifierBudgetRule for PendingVerifierBudgetNameLengthRule {
    fn check(&self, item: &PendingVerifierBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierBudgetRequiredLocaleRule {
    pub message: String,
}

impl PendingVerifierBudgetRule for PendingVerifierBudgetRequiredLocaleRule {
    fn check(&self, item: &PendingVerifierBudget) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending verifier budget.
#[derive(Debug, Clone)]
pub struct PendingVerifierBudgetValidator {
    pub rules: Vec<Rc<dyn PendingVerifierBudgetRule>>,
}

impl PendingVerifierBudgetValidator {
    pub fn validate(&self, item: &PendingVerifierBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingVerifierBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingVerifierBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_verifier_budget_validator() -> PendingVerifierBudgetValidator {
    PendingVerifierBudgetValidator {
        rules: vec![
            Rc::new(PendingVerifierBudgetAllowedNameRule { allowed: Vec::new() }) as Rc<dyn PendingVerifierBudgetRule>,
            Rc::new(PendingVerifierBudgetNameLengthRule { max_length: 86400 }),
            Rc::new(PendingVerifierBudgetRequiredLocaleRule {
                message: "pending verifier budget needs this field".to_string(),
            }),
        ],
    }
}
