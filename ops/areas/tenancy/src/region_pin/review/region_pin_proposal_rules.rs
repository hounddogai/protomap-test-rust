use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin proposal must pass. Check returns its problems, or none.
pub trait RegionPinProposalRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposalRequiredTitleRule {
    pub message: String,
}

impl RegionPinProposalRule for RegionPinProposalRequiredTitleRule {
    fn check(&self, item: &RegionPinProposal) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposalCodePrefixRule {
    pub prefix: String,
}

impl RegionPinProposalRule for RegionPinProposalCodePrefixRule {
    fn check(&self, item: &RegionPinProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposalRequiredLabelRule {
    pub message: String,
}

impl RegionPinProposalRule for RegionPinProposalRequiredLabelRule {
    fn check(&self, item: &RegionPinProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposalDescriptionLengthRule {
    pub max_length: i64,
}

impl RegionPinProposalRule for RegionPinProposalDescriptionLengthRule {
    fn check(&self, item: &RegionPinProposal) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposalTitleLengthRule {
    pub max_length: i64,
}

impl RegionPinProposalRule for RegionPinProposalTitleLengthRule {
    fn check(&self, item: &RegionPinProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinProposalRequiredCodeRule {
    pub message: String,
}

impl RegionPinProposalRule for RegionPinProposalRequiredCodeRule {
    fn check(&self, item: &RegionPinProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin proposal.
#[derive(Debug, Clone)]
pub struct RegionPinProposalValidator {
    pub rules: Vec<Rc<dyn RegionPinProposalRule>>,
}

impl RegionPinProposalValidator {
    pub fn validate(&self, item: &RegionPinProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_proposal_validator() -> RegionPinProposalValidator {
    RegionPinProposalValidator {
        rules: vec![
            Rc::new(RegionPinProposalRequiredTitleRule {
                message: "region pin proposal needs this field".to_string(),
            }) as Rc<dyn RegionPinProposalRule>,
            Rc::new(RegionPinProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionPinProposalRequiredLabelRule {
                message: "region pin proposal needs an entry".to_string(),
            }),
            Rc::new(RegionPinProposalDescriptionLengthRule { max_length: 64 }),
            Rc::new(RegionPinProposalTitleLengthRule { max_length: 1000 }),
            Rc::new(RegionPinProposalRequiredCodeRule {
                message: "region pin proposal needs an entry".to_string(),
            }),
        ],
    }
}
