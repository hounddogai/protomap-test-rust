use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval proposal must pass. Check returns its problems, or none.
pub trait ApprovalProposalRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProposalBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalProposalRule for ApprovalProposalBatchSizeRangeRule {
    fn check(&self, item: &ApprovalProposal) -> Vec<String> {
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
pub struct ApprovalProposalRequiredAudienceRule {
    pub message: String,
}

impl ApprovalProposalRule for ApprovalProposalRequiredAudienceRule {
    fn check(&self, item: &ApprovalProposal) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProposalCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalProposalRule for ApprovalProposalCeilingRangeRule {
    fn check(&self, item: &ApprovalProposal) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProposalAudienceLengthRule {
    pub max_length: i64,
}

impl ApprovalProposalRule for ApprovalProposalAudienceLengthRule {
    fn check(&self, item: &ApprovalProposal) -> Vec<String> {
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
pub struct ApprovalProposalUnitLengthRule {
    pub max_length: i64,
}

impl ApprovalProposalRule for ApprovalProposalUnitLengthRule {
    fn check(&self, item: &ApprovalProposal) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProposalRequiredUnitRule {
    pub message: String,
}

impl ApprovalProposalRule for ApprovalProposalRequiredUnitRule {
    fn check(&self, item: &ApprovalProposal) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval proposal.
#[derive(Debug, Clone)]
pub struct ApprovalProposalValidator {
    pub rules: Vec<Rc<dyn ApprovalProposalRule>>,
}

impl ApprovalProposalValidator {
    pub fn validate(&self, item: &ApprovalProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_proposal_validator() -> ApprovalProposalValidator {
    ApprovalProposalValidator {
        rules: vec![
            Rc::new(ApprovalProposalBatchSizeRangeRule { min: 1000, max: 3600 }) as Rc<dyn ApprovalProposalRule>,
            Rc::new(ApprovalProposalRequiredAudienceRule {
                message: "approval proposal needs a value".to_string(),
            }),
            Rc::new(ApprovalProposalCeilingRangeRule { min: 5.7, max: 449.7 }),
            Rc::new(ApprovalProposalAudienceLengthRule { max_length: 120 }),
            Rc::new(ApprovalProposalUnitLengthRule { max_length: 80 }),
            Rc::new(ApprovalProposalRequiredUnitRule {
                message: "approval proposal needs an entry".to_string(),
            }),
        ],
    }
}
