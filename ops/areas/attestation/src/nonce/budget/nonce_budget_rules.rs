use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a nonce budget must pass. Check returns its problems, or none.
pub trait NonceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &NonceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudgetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceBudgetRule for NonceBudgetGraceDaysRangeRule {
    fn check(&self, item: &NonceBudget) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudgetCodePrefixRule {
    pub prefix: String,
}

impl NonceBudgetRule for NonceBudgetCodePrefixRule {
    fn check(&self, item: &NonceBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudgetRequiredDescriptionRule {
    pub message: String,
}

impl NonceBudgetRule for NonceBudgetRequiredDescriptionRule {
    fn check(&self, item: &NonceBudget) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudgetAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl NonceBudgetRule for NonceBudgetAllowedLabelRule {
    fn check(&self, item: &NonceBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudgetDescriptionLengthRule {
    pub max_length: i64,
}

impl NonceBudgetRule for NonceBudgetDescriptionLengthRule {
    fn check(&self, item: &NonceBudget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudgetRequiredCodeRule {
    pub message: String,
}

impl NonceBudgetRule for NonceBudgetRequiredCodeRule {
    fn check(&self, item: &NonceBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce budget.
#[derive(Debug, Clone)]
pub struct NonceBudgetValidator {
    pub rules: Vec<Rc<dyn NonceBudgetRule>>,
}

impl NonceBudgetValidator {
    pub fn validate(&self, item: &NonceBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &NonceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_budget_validator() -> NonceBudgetValidator {
    NonceBudgetValidator {
        rules: vec![
            Rc::new(NonceBudgetGraceDaysRangeRule { min: 16, max: 3600 }) as Rc<dyn NonceBudgetRule>,
            Rc::new(NonceBudgetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(NonceBudgetRequiredDescriptionRule {
                message: "nonce budget needs an entry".to_string(),
            }),
            Rc::new(NonceBudgetAllowedLabelRule {
                allowed: vec!["global".to_string(), "batched".to_string(), "pending".to_string()],
            }),
            Rc::new(NonceBudgetDescriptionLengthRule { max_length: 255 }),
            Rc::new(NonceBudgetRequiredCodeRule {
                message: "nonce budget needs an entry".to_string(),
            }),
        ],
    }
}
