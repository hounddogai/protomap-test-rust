use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential summary must pass. Check returns its problems, or none.
pub trait CredentialSummaryRule: std::fmt::Debug {
    fn check(&self, item: &CredentialSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSummaryRequiredReviewerRule {
    pub message: String,
}

impl CredentialSummaryRule for CredentialSummaryRequiredReviewerRule {
    fn check(&self, item: &CredentialSummary) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSummaryStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialSummaryRule for CredentialSummaryStartsAtRangeRule {
    fn check(&self, item: &CredentialSummary) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSummaryCodePrefixRule {
    pub prefix: String,
}

impl CredentialSummaryRule for CredentialSummaryCodePrefixRule {
    fn check(&self, item: &CredentialSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential summary.
#[derive(Debug, Clone)]
pub struct CredentialSummaryValidator {
    pub rules: Vec<Rc<dyn CredentialSummaryRule>>,
}

impl CredentialSummaryValidator {
    pub fn validate(&self, item: &CredentialSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_summary_validator() -> CredentialSummaryValidator {
    CredentialSummaryValidator {
        rules: vec![
            Rc::new(CredentialSummaryRequiredReviewerRule {
                message: "credential summary needs an entry".to_string(),
            }) as Rc<dyn CredentialSummaryRule>,
            Rc::new(CredentialSummaryStartsAtRangeRule { min: 120, max: 255 }),
            Rc::new(CredentialSummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
