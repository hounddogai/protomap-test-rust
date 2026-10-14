use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation proposal must pass. Check returns its problems, or none.
pub trait FederationProposalRule: std::fmt::Debug {
    fn check(&self, item: &FederationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationProposalRule for FederationProposalExpiresAtRangeRule {
    fn check(&self, item: &FederationProposal) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationProposalRule for FederationProposalRotationDaysRangeRule {
    fn check(&self, item: &FederationProposal) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalRequiredTitleRule {
    pub message: String,
}

impl FederationProposalRule for FederationProposalRequiredTitleRule {
    fn check(&self, item: &FederationProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalCodePrefixRule {
    pub prefix: String,
}

impl FederationProposalRule for FederationProposalCodePrefixRule {
    fn check(&self, item: &FederationProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalCodeLengthRule {
    pub max_length: i64,
}

impl FederationProposalRule for FederationProposalCodeLengthRule {
    fn check(&self, item: &FederationProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalRequiredFingerprintRule {
    pub message: String,
}

impl FederationProposalRule for FederationProposalRequiredFingerprintRule {
    fn check(&self, item: &FederationProposal) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation proposal.
#[derive(Debug, Clone)]
pub struct FederationProposalValidator {
    pub rules: Vec<Rc<dyn FederationProposalRule>>,
}

impl FederationProposalValidator {
    pub fn validate(&self, item: &FederationProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_proposal_validator() -> FederationProposalValidator {
    FederationProposalValidator {
        rules: vec![
            Rc::new(FederationProposalExpiresAtRangeRule { min: 16, max: 1000 }) as Rc<dyn FederationProposalRule>,
            Rc::new(FederationProposalRotationDaysRangeRule { min: 16, max: 80 }),
            Rc::new(FederationProposalRequiredTitleRule {
                message: "federation proposal needs this field".to_string(),
            }),
            Rc::new(FederationProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FederationProposalCodeLengthRule { max_length: 64 }),
            Rc::new(FederationProposalRequiredFingerprintRule {
                message: "federation proposal needs an entry".to_string(),
            }),
        ],
    }
}
