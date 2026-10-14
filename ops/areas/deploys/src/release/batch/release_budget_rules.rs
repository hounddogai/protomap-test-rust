use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release budget must pass. Check returns its problems, or none.
pub trait ReleaseBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBudgetPinnedGuardRule {
    pub limit: f64,
}

impl ReleaseBudgetRule for ReleaseBudgetPinnedGuardRule {
    fn check(&self, item: &ReleaseBudget) -> Vec<String> {
        if item.pinned && (item.maximum > self.limit) {
            return vec![format!("maximum {} exceeds {} while pinned", item.maximum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBudgetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ReleaseBudgetRule for ReleaseBudgetAllowedTitleRule {
    fn check(&self, item: &ReleaseBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBudgetRequiredTitleRule {
    pub message: String,
}

impl ReleaseBudgetRule for ReleaseBudgetRequiredTitleRule {
    fn check(&self, item: &ReleaseBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBudgetLabelLengthRule {
    pub max_length: i64,
}

impl ReleaseBudgetRule for ReleaseBudgetLabelLengthRule {
    fn check(&self, item: &ReleaseBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release budget.
#[derive(Debug, Clone)]
pub struct ReleaseBudgetValidator {
    pub rules: Vec<Rc<dyn ReleaseBudgetRule>>,
}

impl ReleaseBudgetValidator {
    pub fn validate(&self, item: &ReleaseBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_budget_validator() -> ReleaseBudgetValidator {
    ReleaseBudgetValidator {
        rules: vec![
            Rc::new(ReleaseBudgetPinnedGuardRule { limit: 4.9 }) as Rc<dyn ReleaseBudgetRule>,
            Rc::new(ReleaseBudgetAllowedTitleRule {
                allowed: vec!["rolling".to_string(), "adaptive".to_string()],
            }),
            Rc::new(ReleaseBudgetRequiredTitleRule {
                message: "release budget needs this field".to_string(),
            }),
            Rc::new(ReleaseBudgetLabelLengthRule { max_length: 32 }),
        ],
    }
}
