use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule proposal must pass. Check returns its problems, or none.
pub trait RuleProposalRule: std::fmt::Debug {
    fn check(&self, item: &RuleProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProposalSegmentLengthRule {
    pub max_length: i64,
}

impl RuleProposalRule for RuleProposalSegmentLengthRule {
    fn check(&self, item: &RuleProposal) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProposalRequiredTitleRule {
    pub message: String,
}

impl RuleProposalRule for RuleProposalRequiredTitleRule {
    fn check(&self, item: &RuleProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProposalRequiredSegmentRule {
    pub message: String,
}

impl RuleProposalRule for RuleProposalRequiredSegmentRule {
    fn check(&self, item: &RuleProposal) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProposalMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleProposalRule for RuleProposalMaximumRangeRule {
    fn check(&self, item: &RuleProposal) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleProposalFingerprintLengthRule {
    pub max_length: i64,
}

impl RuleProposalRule for RuleProposalFingerprintLengthRule {
    fn check(&self, item: &RuleProposal) -> Vec<String> {
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

/// Runs every rule of a rule proposal.
#[derive(Debug, Clone)]
pub struct RuleProposalValidator {
    pub rules: Vec<Rc<dyn RuleProposalRule>>,
}

impl RuleProposalValidator {
    pub fn validate(&self, item: &RuleProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_proposal_validator() -> RuleProposalValidator {
    RuleProposalValidator {
        rules: vec![
            Rc::new(RuleProposalSegmentLengthRule { max_length: 16 }) as Rc<dyn RuleProposalRule>,
            Rc::new(RuleProposalRequiredTitleRule {
                message: "rule proposal needs this field".to_string(),
            }),
            Rc::new(RuleProposalRequiredSegmentRule {
                message: "rule proposal needs this field".to_string(),
            }),
            Rc::new(RuleProposalMaximumRangeRule { min: 5.6, max: 1254.2 }),
            Rc::new(RuleProposalFingerprintLengthRule { max_length: 16 }),
        ],
    }
}
