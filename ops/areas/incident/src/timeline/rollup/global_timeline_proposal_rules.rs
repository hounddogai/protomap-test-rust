use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global timeline proposal must pass. Check returns its problems, or none.
pub trait GlobalTimelineProposalRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTimelineProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalTimelineProposalRule for GlobalTimelineProposalDiscountRangeRule {
    fn check(&self, item: &GlobalTimelineProposal) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalRequiredNameRule {
    pub message: String,
}

impl GlobalTimelineProposalRule for GlobalTimelineProposalRequiredNameRule {
    fn check(&self, item: &GlobalTimelineProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalSourceLengthRule {
    pub max_length: i64,
}

impl GlobalTimelineProposalRule for GlobalTimelineProposalSourceLengthRule {
    fn check(&self, item: &GlobalTimelineProposal) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalUnitLengthRule {
    pub max_length: i64,
}

impl GlobalTimelineProposalRule for GlobalTimelineProposalUnitLengthRule {
    fn check(&self, item: &GlobalTimelineProposal) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl GlobalTimelineProposalRule for GlobalTimelineProposalTagsLimitRule {
    fn check(&self, item: &GlobalTimelineProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalCodePrefixRule {
    pub prefix: String,
}

impl GlobalTimelineProposalRule for GlobalTimelineProposalCodePrefixRule {
    fn check(&self, item: &GlobalTimelineProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global timeline proposal.
#[derive(Debug, Clone)]
pub struct GlobalTimelineProposalValidator {
    pub rules: Vec<Rc<dyn GlobalTimelineProposalRule>>,
}

impl GlobalTimelineProposalValidator {
    pub fn validate(&self, item: &GlobalTimelineProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalTimelineProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTimelineProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_timeline_proposal_validator() -> GlobalTimelineProposalValidator {
    GlobalTimelineProposalValidator {
        rules: vec![
            Rc::new(GlobalTimelineProposalDiscountRangeRule { min: 4.3, max: 3910.1 })
                as Rc<dyn GlobalTimelineProposalRule>,
            Rc::new(GlobalTimelineProposalRequiredNameRule {
                message: "global timeline proposal needs an entry".to_string(),
            }),
            Rc::new(GlobalTimelineProposalSourceLengthRule { max_length: 86400 }),
            Rc::new(GlobalTimelineProposalUnitLengthRule { max_length: 120 }),
            Rc::new(GlobalTimelineProposalTagsLimitRule {
                max_count: 1000,
                banned: Vec::new(),
            }),
            Rc::new(GlobalTimelineProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
