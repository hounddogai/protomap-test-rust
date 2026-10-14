use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval budget must pass. Check returns its problems, or none.
pub trait ApprovalBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBudgetSegmentLengthRule {
    pub max_length: i64,
}

impl ApprovalBudgetRule for ApprovalBudgetSegmentLengthRule {
    fn check(&self, item: &ApprovalBudget) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBudgetRequiredLabelRule {
    pub message: String,
}

impl ApprovalBudgetRule for ApprovalBudgetRequiredLabelRule {
    fn check(&self, item: &ApprovalBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBudgetRequiredSegmentRule {
    pub message: String,
}

impl ApprovalBudgetRule for ApprovalBudgetRequiredSegmentRule {
    fn check(&self, item: &ApprovalBudget) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval budget.
#[derive(Debug, Clone)]
pub struct ApprovalBudgetValidator {
    pub rules: Vec<Rc<dyn ApprovalBudgetRule>>,
}

impl ApprovalBudgetValidator {
    pub fn validate(&self, item: &ApprovalBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_budget_validator() -> ApprovalBudgetValidator {
    ApprovalBudgetValidator {
        rules: vec![
            Rc::new(ApprovalBudgetSegmentLengthRule { max_length: 16 }) as Rc<dyn ApprovalBudgetRule>,
            Rc::new(ApprovalBudgetRequiredLabelRule {
                message: "approval budget needs an entry".to_string(),
            }),
            Rc::new(ApprovalBudgetRequiredSegmentRule {
                message: "approval budget needs an entry".to_string(),
            }),
        ],
    }
}
