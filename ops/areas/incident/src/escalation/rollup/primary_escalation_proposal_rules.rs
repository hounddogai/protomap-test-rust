use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary escalation proposal must pass. Check returns its problems, or none.
pub trait PrimaryEscalationProposalRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEscalationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationProposalGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEscalationProposalRule for PrimaryEscalationProposalGraceDaysRangeRule {
    fn check(&self, item: &PrimaryEscalationProposal) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationProposalCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEscalationProposalRule for PrimaryEscalationProposalCodePrefixRule {
    fn check(&self, item: &PrimaryEscalationProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationProposalAudienceLengthRule {
    pub max_length: i64,
}

impl PrimaryEscalationProposalRule for PrimaryEscalationProposalAudienceLengthRule {
    fn check(&self, item: &PrimaryEscalationProposal) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationProposalRequiredFingerprintRule {
    pub message: String,
}

impl PrimaryEscalationProposalRule for PrimaryEscalationProposalRequiredFingerprintRule {
    fn check(&self, item: &PrimaryEscalationProposal) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary escalation proposal.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationProposalValidator {
    pub rules: Vec<Rc<dyn PrimaryEscalationProposalRule>>,
}

impl PrimaryEscalationProposalValidator {
    pub fn validate(&self, item: &PrimaryEscalationProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryEscalationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEscalationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_escalation_proposal_validator() -> PrimaryEscalationProposalValidator {
    PrimaryEscalationProposalValidator {
        rules: vec![
            Rc::new(PrimaryEscalationProposalGraceDaysRangeRule { min: 64, max: 120 })
                as Rc<dyn PrimaryEscalationProposalRule>,
            Rc::new(PrimaryEscalationProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryEscalationProposalAudienceLengthRule { max_length: 64 }),
            Rc::new(PrimaryEscalationProposalRequiredFingerprintRule {
                message: "primary escalation proposal needs a value".to_string(),
            }),
        ],
    }
}
