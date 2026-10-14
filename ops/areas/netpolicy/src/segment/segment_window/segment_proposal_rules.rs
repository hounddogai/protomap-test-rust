use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment proposal must pass. Check returns its problems, or none.
pub trait SegmentProposalRule: std::fmt::Debug {
    fn check(&self, item: &SegmentProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProposalTimezoneLengthRule {
    pub max_length: i64,
}

impl SegmentProposalRule for SegmentProposalTimezoneLengthRule {
    fn check(&self, item: &SegmentProposal) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProposalBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentProposalRule for SegmentProposalBatchSizeRangeRule {
    fn check(&self, item: &SegmentProposal) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProposalAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl SegmentProposalRule for SegmentProposalAllowedTimezoneRule {
    fn check(&self, item: &SegmentProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is too long", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProposalRequiredCipherRule {
    pub message: String,
}

impl SegmentProposalRule for SegmentProposalRequiredCipherRule {
    fn check(&self, item: &SegmentProposal) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProposalCipherLengthRule {
    pub max_length: i64,
}

impl SegmentProposalRule for SegmentProposalCipherLengthRule {
    fn check(&self, item: &SegmentProposal) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment proposal.
#[derive(Debug, Clone)]
pub struct SegmentProposalValidator {
    pub rules: Vec<Rc<dyn SegmentProposalRule>>,
}

impl SegmentProposalValidator {
    pub fn validate(&self, item: &SegmentProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_proposal_validator() -> SegmentProposalValidator {
    SegmentProposalValidator {
        rules: vec![
            Rc::new(SegmentProposalTimezoneLengthRule { max_length: 3600 }) as Rc<dyn SegmentProposalRule>,
            Rc::new(SegmentProposalBatchSizeRangeRule { min: 64, max: 120 }),
            Rc::new(SegmentProposalAllowedTimezoneRule {
                allowed: vec!["batched".to_string(), "default".to_string()],
            }),
            Rc::new(SegmentProposalRequiredCipherRule {
                message: "segment proposal needs an entry".to_string(),
            }),
            Rc::new(SegmentProposalCipherLengthRule { max_length: 120 }),
        ],
    }
}
