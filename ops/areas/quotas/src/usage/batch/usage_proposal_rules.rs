use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage proposal must pass. Check returns its problems, or none.
pub trait UsageProposalRule: std::fmt::Debug {
    fn check(&self, item: &UsageProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposalAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl UsageProposalRule for UsageProposalAllowedChecksumRule {
    fn check(&self, item: &UsageProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is blocked", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposalTitleLengthRule {
    pub max_length: i64,
}

impl UsageProposalRule for UsageProposalTitleLengthRule {
    fn check(&self, item: &UsageProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposalRequiredPrincipalRule {
    pub message: String,
}

impl UsageProposalRule for UsageProposalRequiredPrincipalRule {
    fn check(&self, item: &UsageProposal) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposalCodePrefixRule {
    pub prefix: String,
}

impl UsageProposalRule for UsageProposalCodePrefixRule {
    fn check(&self, item: &UsageProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposalApprovedGuardRule {
    pub limit: i64,
}

impl UsageProposalRule for UsageProposalApprovedGuardRule {
    fn check(&self, item: &UsageProposal) -> Vec<String> {
        if item.approved && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while approved", item.port, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage proposal.
#[derive(Debug, Clone)]
pub struct UsageProposalValidator {
    pub rules: Vec<Rc<dyn UsageProposalRule>>,
}

impl UsageProposalValidator {
    pub fn validate(&self, item: &UsageProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_proposal_validator() -> UsageProposalValidator {
    UsageProposalValidator {
        rules: vec![
            Rc::new(UsageProposalAllowedChecksumRule {
                allowed: vec!["rolling".to_string(), "scheduled".to_string(), "manual".to_string()],
            }) as Rc<dyn UsageProposalRule>,
            Rc::new(UsageProposalTitleLengthRule { max_length: 1000 }),
            Rc::new(UsageProposalRequiredPrincipalRule {
                message: "usage proposal needs this field".to_string(),
            }),
            Rc::new(UsageProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(UsageProposalApprovedGuardRule { limit: 1000 }),
        ],
    }
}
