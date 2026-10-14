use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary verifier quota must pass. Check returns its problems, or none.
pub trait PrimaryVerifierQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryVerifierQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierQuotaRequiredTitleRule {
    pub message: String,
}

impl PrimaryVerifierQuotaRule for PrimaryVerifierQuotaRequiredTitleRule {
    fn check(&self, item: &PrimaryVerifierQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierQuotaRequiredCodeRule {
    pub message: String,
}

impl PrimaryVerifierQuotaRule for PrimaryVerifierQuotaRequiredCodeRule {
    fn check(&self, item: &PrimaryVerifierQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierQuotaReviewerLengthRule {
    pub max_length: i64,
}

impl PrimaryVerifierQuotaRule for PrimaryVerifierQuotaReviewerLengthRule {
    fn check(&self, item: &PrimaryVerifierQuota) -> Vec<String> {
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
pub struct PrimaryVerifierQuotaCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryVerifierQuotaRule for PrimaryVerifierQuotaCodeLengthRule {
    fn check(&self, item: &PrimaryVerifierQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary verifier quota.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierQuotaValidator {
    pub rules: Vec<Rc<dyn PrimaryVerifierQuotaRule>>,
}

impl PrimaryVerifierQuotaValidator {
    pub fn validate(&self, item: &PrimaryVerifierQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryVerifierQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryVerifierQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_verifier_quota_validator() -> PrimaryVerifierQuotaValidator {
    PrimaryVerifierQuotaValidator {
        rules: vec![
            Rc::new(PrimaryVerifierQuotaRequiredTitleRule {
                message: "primary verifier quota needs a value".to_string(),
            }) as Rc<dyn PrimaryVerifierQuotaRule>,
            Rc::new(PrimaryVerifierQuotaRequiredCodeRule {
                message: "primary verifier quota needs a value".to_string(),
            }),
            Rc::new(PrimaryVerifierQuotaReviewerLengthRule { max_length: 64 }),
            Rc::new(PrimaryVerifierQuotaCodeLengthRule { max_length: 16 }),
        ],
    }
}
