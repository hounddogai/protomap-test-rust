use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding draft must pass. Check returns its problems, or none.
pub trait BindingDraftRule: std::fmt::Debug {
    fn check(&self, item: &BindingDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingDraftCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingDraftRule for BindingDraftCostRangeRule {
    fn check(&self, item: &BindingDraft) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingDraftCodePrefixRule {
    pub prefix: String,
}

impl BindingDraftRule for BindingDraftCodePrefixRule {
    fn check(&self, item: &BindingDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingDraftAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BindingDraftRule for BindingDraftAllowedNameRule {
    fn check(&self, item: &BindingDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding draft.
#[derive(Debug, Clone)]
pub struct BindingDraftValidator {
    pub rules: Vec<Rc<dyn BindingDraftRule>>,
}

impl BindingDraftValidator {
    pub fn validate(&self, item: &BindingDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_draft_validator() -> BindingDraftValidator {
    BindingDraftValidator {
        rules: vec![
            Rc::new(BindingDraftCostRangeRule { min: 4.5, max: 3530.2 }) as Rc<dyn BindingDraftRule>,
            Rc::new(BindingDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BindingDraftAllowedNameRule {
                allowed: vec!["adaptive".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}
