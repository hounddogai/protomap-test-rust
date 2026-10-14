use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive alert budget must pass. Check returns its problems, or none.
pub trait AdaptiveAlertBudgetRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAlertBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertBudgetCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveAlertBudgetRule for AdaptiveAlertBudgetCodePrefixRule {
    fn check(&self, item: &AdaptiveAlertBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertBudgetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveAlertBudgetRule for AdaptiveAlertBudgetAllowedCodeRule {
    fn check(&self, item: &AdaptiveAlertBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertBudgetRequiredSummaryRule {
    pub message: String,
}

impl AdaptiveAlertBudgetRule for AdaptiveAlertBudgetRequiredSummaryRule {
    fn check(&self, item: &AdaptiveAlertBudget) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertBudgetNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveAlertBudgetRule for AdaptiveAlertBudgetNameLengthRule {
    fn check(&self, item: &AdaptiveAlertBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive alert budget.
#[derive(Debug, Clone)]
pub struct AdaptiveAlertBudgetValidator {
    pub rules: Vec<Rc<dyn AdaptiveAlertBudgetRule>>,
}

impl AdaptiveAlertBudgetValidator {
    pub fn validate(&self, item: &AdaptiveAlertBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveAlertBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAlertBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_alert_budget_validator() -> AdaptiveAlertBudgetValidator {
    AdaptiveAlertBudgetValidator {
        rules: vec![
            Rc::new(AdaptiveAlertBudgetCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn AdaptiveAlertBudgetRule>,
            Rc::new(AdaptiveAlertBudgetAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(AdaptiveAlertBudgetRequiredSummaryRule {
                message: "adaptive alert budget needs an entry".to_string(),
            }),
            Rc::new(AdaptiveAlertBudgetNameLengthRule { max_length: 86400 }),
        ],
    }
}
