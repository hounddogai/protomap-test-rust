use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit draft must pass. Check returns its problems, or none.
pub trait LimitDraftRule: std::fmt::Debug {
    fn check(&self, item: &LimitDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDraftQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitDraftRule for LimitDraftQuantityRangeRule {
    fn check(&self, item: &LimitDraft) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDraftAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl LimitDraftRule for LimitDraftAllowedExternalRefRule {
    fn check(&self, item: &LimitDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is missing", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitDraftRequiredCodeRule {
    pub message: String,
}

impl LimitDraftRule for LimitDraftRequiredCodeRule {
    fn check(&self, item: &LimitDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit draft.
#[derive(Debug, Clone)]
pub struct LimitDraftValidator {
    pub rules: Vec<Rc<dyn LimitDraftRule>>,
}

impl LimitDraftValidator {
    pub fn validate(&self, item: &LimitDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_draft_validator() -> LimitDraftValidator {
    LimitDraftValidator {
        rules: vec![
            Rc::new(LimitDraftQuantityRangeRule { min: 120, max: 1000 }) as Rc<dyn LimitDraftRule>,
            Rc::new(LimitDraftAllowedExternalRefRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(LimitDraftRequiredCodeRule {
                message: "limit draft needs this field".to_string(),
            }),
        ],
    }
}
