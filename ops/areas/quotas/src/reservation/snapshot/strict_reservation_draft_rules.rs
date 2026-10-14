use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict reservation draft must pass. Check returns its problems, or none.
pub trait StrictReservationDraftRule: std::fmt::Debug {
    fn check(&self, item: &StrictReservationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationDraftCodePrefixRule {
    pub prefix: String,
}

impl StrictReservationDraftRule for StrictReservationDraftCodePrefixRule {
    fn check(&self, item: &StrictReservationDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationDraftBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictReservationDraftRule for StrictReservationDraftBudgetRangeRule {
    fn check(&self, item: &StrictReservationDraft) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationDraftCodeLengthRule {
    pub max_length: i64,
}

impl StrictReservationDraftRule for StrictReservationDraftCodeLengthRule {
    fn check(&self, item: &StrictReservationDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict reservation draft.
#[derive(Debug, Clone)]
pub struct StrictReservationDraftValidator {
    pub rules: Vec<Rc<dyn StrictReservationDraftRule>>,
}

impl StrictReservationDraftValidator {
    pub fn validate(&self, item: &StrictReservationDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictReservationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictReservationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_reservation_draft_validator() -> StrictReservationDraftValidator {
    StrictReservationDraftValidator {
        rules: vec![
            Rc::new(StrictReservationDraftCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn StrictReservationDraftRule>,
            Rc::new(StrictReservationDraftBudgetRangeRule { min: 0.5, max: 4097.7 }),
            Rc::new(StrictReservationDraftCodeLengthRule { max_length: 80 }),
        ],
    }
}
