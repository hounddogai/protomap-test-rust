use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback segment proposal must pass. Check returns its problems, or none.
pub trait FallbackSegmentProposalRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSegmentProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentProposalSummaryLengthRule {
    pub max_length: i64,
}

impl FallbackSegmentProposalRule for FallbackSegmentProposalSummaryLengthRule {
    fn check(&self, item: &FallbackSegmentProposal) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentProposalRequiredCodeRule {
    pub message: String,
}

impl FallbackSegmentProposalRule for FallbackSegmentProposalRequiredCodeRule {
    fn check(&self, item: &FallbackSegmentProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentProposalCodeLengthRule {
    pub max_length: i64,
}

impl FallbackSegmentProposalRule for FallbackSegmentProposalCodeLengthRule {
    fn check(&self, item: &FallbackSegmentProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentProposalRequiredFingerprintRule {
    pub message: String,
}

impl FallbackSegmentProposalRule for FallbackSegmentProposalRequiredFingerprintRule {
    fn check(&self, item: &FallbackSegmentProposal) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentProposalFingerprintLengthRule {
    pub max_length: i64,
}

impl FallbackSegmentProposalRule for FallbackSegmentProposalFingerprintLengthRule {
    fn check(&self, item: &FallbackSegmentProposal) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback segment proposal.
#[derive(Debug, Clone)]
pub struct FallbackSegmentProposalValidator {
    pub rules: Vec<Rc<dyn FallbackSegmentProposalRule>>,
}

impl FallbackSegmentProposalValidator {
    pub fn validate(&self, item: &FallbackSegmentProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackSegmentProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSegmentProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_segment_proposal_validator() -> FallbackSegmentProposalValidator {
    FallbackSegmentProposalValidator {
        rules: vec![
            Rc::new(FallbackSegmentProposalSummaryLengthRule { max_length: 64 }) as Rc<dyn FallbackSegmentProposalRule>,
            Rc::new(FallbackSegmentProposalRequiredCodeRule {
                message: "fallback segment proposal needs a value".to_string(),
            }),
            Rc::new(FallbackSegmentProposalCodeLengthRule { max_length: 32 }),
            Rc::new(FallbackSegmentProposalRequiredFingerprintRule {
                message: "fallback segment proposal needs this field".to_string(),
            }),
            Rc::new(FallbackSegmentProposalFingerprintLengthRule { max_length: 3600 }),
        ],
    }
}
