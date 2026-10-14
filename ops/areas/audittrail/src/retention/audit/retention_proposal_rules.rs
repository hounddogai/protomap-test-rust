use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention proposal must pass. Check returns its problems, or none.
pub trait RetentionProposalRule: std::fmt::Debug {
    fn check(&self, item: &RetentionProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProposalRequiredChannelRule {
    pub message: String,
}

impl RetentionProposalRule for RetentionProposalRequiredChannelRule {
    fn check(&self, item: &RetentionProposal) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProposalWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionProposalRule for RetentionProposalWeightRangeRule {
    fn check(&self, item: &RetentionProposal) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProposalRequiredCodeRule {
    pub message: String,
}

impl RetentionProposalRule for RetentionProposalRequiredCodeRule {
    fn check(&self, item: &RetentionProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProposalPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionProposalRule for RetentionProposalPriorityRangeRule {
    fn check(&self, item: &RetentionProposal) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProposalCodePrefixRule {
    pub prefix: String,
}

impl RetentionProposalRule for RetentionProposalCodePrefixRule {
    fn check(&self, item: &RetentionProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionProposalTitleLengthRule {
    pub max_length: i64,
}

impl RetentionProposalRule for RetentionProposalTitleLengthRule {
    fn check(&self, item: &RetentionProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention proposal.
#[derive(Debug, Clone)]
pub struct RetentionProposalValidator {
    pub rules: Vec<Rc<dyn RetentionProposalRule>>,
}

impl RetentionProposalValidator {
    pub fn validate(&self, item: &RetentionProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_proposal_validator() -> RetentionProposalValidator {
    RetentionProposalValidator {
        rules: vec![
            Rc::new(RetentionProposalRequiredChannelRule {
                message: "retention proposal needs a value".to_string(),
            }) as Rc<dyn RetentionProposalRule>,
            Rc::new(RetentionProposalWeightRangeRule { min: 2.6, max: 3623.8 }),
            Rc::new(RetentionProposalRequiredCodeRule {
                message: "retention proposal needs a value".to_string(),
            }),
            Rc::new(RetentionProposalPriorityRangeRule { min: 80, max: 86400 }),
            Rc::new(RetentionProposalCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RetentionProposalTitleLengthRule { max_length: 86400 }),
        ],
    }
}
