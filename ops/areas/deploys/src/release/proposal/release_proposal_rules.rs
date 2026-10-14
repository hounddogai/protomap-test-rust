use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release proposal must pass. Check returns its problems, or none.
pub trait ReleaseProposalRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposalCodePrefixRule {
    pub prefix: String,
}

impl ReleaseProposalRule for ReleaseProposalCodePrefixRule {
    fn check(&self, item: &ReleaseProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposalNameLengthRule {
    pub max_length: i64,
}

impl ReleaseProposalRule for ReleaseProposalNameLengthRule {
    fn check(&self, item: &ReleaseProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposalRequiredChecksumRule {
    pub message: String,
}

impl ReleaseProposalRule for ReleaseProposalRequiredChecksumRule {
    fn check(&self, item: &ReleaseProposal) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ReleaseProposalRule for ReleaseProposalAllowedCodeRule {
    fn check(&self, item: &ReleaseProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposalPhoneLengthRule {
    pub max_length: i64,
}

impl ReleaseProposalRule for ReleaseProposalPhoneLengthRule {
    fn check(&self, item: &ReleaseProposal) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposalRequiredPhoneRule {
    pub message: String,
}

impl ReleaseProposalRule for ReleaseProposalRequiredPhoneRule {
    fn check(&self, item: &ReleaseProposal) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release proposal.
#[derive(Debug, Clone)]
pub struct ReleaseProposalValidator {
    pub rules: Vec<Rc<dyn ReleaseProposalRule>>,
}

impl ReleaseProposalValidator {
    pub fn validate(&self, item: &ReleaseProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_proposal_validator() -> ReleaseProposalValidator {
    ReleaseProposalValidator {
        rules: vec![
            Rc::new(ReleaseProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn ReleaseProposalRule>,
            Rc::new(ReleaseProposalNameLengthRule { max_length: 86400 }),
            Rc::new(ReleaseProposalRequiredChecksumRule {
                message: "release proposal needs an entry".to_string(),
            }),
            Rc::new(ReleaseProposalAllowedCodeRule {
                allowed: vec!["primary".to_string(), "batched".to_string(), "seasonal".to_string()],
            }),
            Rc::new(ReleaseProposalPhoneLengthRule { max_length: 64 }),
            Rc::new(ReleaseProposalRequiredPhoneRule {
                message: "release proposal needs an entry".to_string(),
            }),
        ],
    }
}
