use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending runbook estimate must pass. Check returns its problems, or none.
pub trait PendingRunbookEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PendingRunbookEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookEstimateRequiredNamespaceRule {
    pub message: String,
}

impl PendingRunbookEstimateRule for PendingRunbookEstimateRequiredNamespaceRule {
    fn check(&self, item: &PendingRunbookEstimate) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookEstimateRequiredCodeRule {
    pub message: String,
}

impl PendingRunbookEstimateRule for PendingRunbookEstimateRequiredCodeRule {
    fn check(&self, item: &PendingRunbookEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookEstimateRequiredNameRule {
    pub message: String,
}

impl PendingRunbookEstimateRule for PendingRunbookEstimateRequiredNameRule {
    fn check(&self, item: &PendingRunbookEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookEstimateCodePrefixRule {
    pub prefix: String,
}

impl PendingRunbookEstimateRule for PendingRunbookEstimateCodePrefixRule {
    fn check(&self, item: &PendingRunbookEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending runbook estimate.
#[derive(Debug, Clone)]
pub struct PendingRunbookEstimateValidator {
    pub rules: Vec<Rc<dyn PendingRunbookEstimateRule>>,
}

impl PendingRunbookEstimateValidator {
    pub fn validate(&self, item: &PendingRunbookEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRunbookEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRunbookEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_runbook_estimate_validator() -> PendingRunbookEstimateValidator {
    PendingRunbookEstimateValidator {
        rules: vec![
            Rc::new(PendingRunbookEstimateRequiredNamespaceRule {
                message: "pending runbook estimate needs an entry".to_string(),
            }) as Rc<dyn PendingRunbookEstimateRule>,
            Rc::new(PendingRunbookEstimateRequiredCodeRule {
                message: "pending runbook estimate needs an entry".to_string(),
            }),
            Rc::new(PendingRunbookEstimateRequiredNameRule {
                message: "pending runbook estimate needs an entry".to_string(),
            }),
            Rc::new(PendingRunbookEstimateCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
