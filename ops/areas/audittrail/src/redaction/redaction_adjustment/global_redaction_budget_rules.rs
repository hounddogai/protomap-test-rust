use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global redaction budget must pass. Check returns its problems, or none.
pub trait GlobalRedactionBudgetRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRedactionBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionBudgetReviewerLengthRule {
    pub max_length: i64,
}

impl GlobalRedactionBudgetRule for GlobalRedactionBudgetReviewerLengthRule {
    fn check(&self, item: &GlobalRedactionBudget) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionBudgetRequiredCodeRule {
    pub message: String,
}

impl GlobalRedactionBudgetRule for GlobalRedactionBudgetRequiredCodeRule {
    fn check(&self, item: &GlobalRedactionBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionBudgetRequiredLabelRule {
    pub message: String,
}

impl GlobalRedactionBudgetRule for GlobalRedactionBudgetRequiredLabelRule {
    fn check(&self, item: &GlobalRedactionBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionBudgetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalRedactionBudgetRule for GlobalRedactionBudgetCeilingRangeRule {
    fn check(&self, item: &GlobalRedactionBudget) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionBudgetRequiredReviewerRule {
    pub message: String,
}

impl GlobalRedactionBudgetRule for GlobalRedactionBudgetRequiredReviewerRule {
    fn check(&self, item: &GlobalRedactionBudget) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionBudgetCodePrefixRule {
    pub prefix: String,
}

impl GlobalRedactionBudgetRule for GlobalRedactionBudgetCodePrefixRule {
    fn check(&self, item: &GlobalRedactionBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global redaction budget.
#[derive(Debug, Clone)]
pub struct GlobalRedactionBudgetValidator {
    pub rules: Vec<Rc<dyn GlobalRedactionBudgetRule>>,
}

impl GlobalRedactionBudgetValidator {
    pub fn validate(&self, item: &GlobalRedactionBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalRedactionBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRedactionBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_redaction_budget_validator() -> GlobalRedactionBudgetValidator {
    GlobalRedactionBudgetValidator {
        rules: vec![
            Rc::new(GlobalRedactionBudgetReviewerLengthRule { max_length: 1000 }) as Rc<dyn GlobalRedactionBudgetRule>,
            Rc::new(GlobalRedactionBudgetRequiredCodeRule {
                message: "global redaction budget needs this field".to_string(),
            }),
            Rc::new(GlobalRedactionBudgetRequiredLabelRule {
                message: "global redaction budget needs an entry".to_string(),
            }),
            Rc::new(GlobalRedactionBudgetCeilingRangeRule { min: 0.5, max: 2416.5 }),
            Rc::new(GlobalRedactionBudgetRequiredReviewerRule {
                message: "global redaction budget needs a value".to_string(),
            }),
            Rc::new(GlobalRedactionBudgetCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
