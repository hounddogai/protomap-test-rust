use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending retention proposal must pass. Check returns its problems, or none.
pub trait PendingRetentionProposalRule: std::fmt::Debug {
    fn check(&self, item: &PendingRetentionProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionProposalRequiredContactEmailRule {
    pub message: String,
}

impl PendingRetentionProposalRule for PendingRetentionProposalRequiredContactEmailRule {
    fn check(&self, item: &PendingRetentionProposal) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionProposalContactEmailLengthRule {
    pub max_length: i64,
}

impl PendingRetentionProposalRule for PendingRetentionProposalContactEmailLengthRule {
    fn check(&self, item: &PendingRetentionProposal) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionProposalCodeLengthRule {
    pub max_length: i64,
}

impl PendingRetentionProposalRule for PendingRetentionProposalCodeLengthRule {
    fn check(&self, item: &PendingRetentionProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending retention proposal.
#[derive(Debug, Clone)]
pub struct PendingRetentionProposalValidator {
    pub rules: Vec<Rc<dyn PendingRetentionProposalRule>>,
}

impl PendingRetentionProposalValidator {
    pub fn validate(&self, item: &PendingRetentionProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRetentionProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRetentionProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_retention_proposal_validator() -> PendingRetentionProposalValidator {
    PendingRetentionProposalValidator {
        rules: vec![
            Rc::new(PendingRetentionProposalRequiredContactEmailRule {
                message: "pending retention proposal needs a value".to_string(),
            }) as Rc<dyn PendingRetentionProposalRule>,
            Rc::new(PendingRetentionProposalContactEmailLengthRule { max_length: 64 }),
            Rc::new(PendingRetentionProposalCodeLengthRule { max_length: 1000 }),
        ],
    }
}
