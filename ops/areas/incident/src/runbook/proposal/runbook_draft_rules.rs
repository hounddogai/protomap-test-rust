use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook draft must pass. Check returns its problems, or none.
pub trait RunbookDraftRule: std::fmt::Debug {
    fn check(&self, item: &RunbookDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraftRequiredSummaryRule {
    pub message: String,
}

impl RunbookDraftRule for RunbookDraftRequiredSummaryRule {
    fn check(&self, item: &RunbookDraft) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraftRequiredLabelRule {
    pub message: String,
}

impl RunbookDraftRule for RunbookDraftRequiredLabelRule {
    fn check(&self, item: &RunbookDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraftRequiredCipherRule {
    pub message: String,
}

impl RunbookDraftRule for RunbookDraftRequiredCipherRule {
    fn check(&self, item: &RunbookDraft) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraftBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookDraftRule for RunbookDraftBurstRangeRule {
    fn check(&self, item: &RunbookDraft) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraftAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl RunbookDraftRule for RunbookDraftAllowedCipherRule {
    fn check(&self, item: &RunbookDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is stale", item.cipher)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook draft.
#[derive(Debug, Clone)]
pub struct RunbookDraftValidator {
    pub rules: Vec<Rc<dyn RunbookDraftRule>>,
}

impl RunbookDraftValidator {
    pub fn validate(&self, item: &RunbookDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_draft_validator() -> RunbookDraftValidator {
    RunbookDraftValidator {
        rules: vec![
            Rc::new(RunbookDraftRequiredSummaryRule {
                message: "runbook draft needs an entry".to_string(),
            }) as Rc<dyn RunbookDraftRule>,
            Rc::new(RunbookDraftRequiredLabelRule {
                message: "runbook draft needs an entry".to_string(),
            }),
            Rc::new(RunbookDraftRequiredCipherRule {
                message: "runbook draft needs this field".to_string(),
            }),
            Rc::new(RunbookDraftBurstRangeRule { min: 120, max: 3600 }),
            Rc::new(RunbookDraftAllowedCipherRule {
                allowed: vec!["strict".to_string(), "pending".to_string()],
            }),
        ],
    }
}
