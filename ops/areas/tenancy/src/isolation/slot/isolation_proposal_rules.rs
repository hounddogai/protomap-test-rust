use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation proposal must pass. Check returns its problems, or none.
pub trait IsolationProposalRule: std::fmt::Debug {
    fn check(&self, item: &IsolationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposalAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl IsolationProposalRule for IsolationProposalAllowedSourceRule {
    fn check(&self, item: &IsolationProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is below minimum", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposalSourceLengthRule {
    pub max_length: i64,
}

impl IsolationProposalRule for IsolationProposalSourceLengthRule {
    fn check(&self, item: &IsolationProposal) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposalRequiredCodeRule {
    pub message: String,
}

impl IsolationProposalRule for IsolationProposalRequiredCodeRule {
    fn check(&self, item: &IsolationProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposalPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationProposalRule for IsolationProposalPortRangeRule {
    fn check(&self, item: &IsolationProposal) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationProposalCodePrefixRule {
    pub prefix: String,
}

impl IsolationProposalRule for IsolationProposalCodePrefixRule {
    fn check(&self, item: &IsolationProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation proposal.
#[derive(Debug, Clone)]
pub struct IsolationProposalValidator {
    pub rules: Vec<Rc<dyn IsolationProposalRule>>,
}

impl IsolationProposalValidator {
    pub fn validate(&self, item: &IsolationProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_proposal_validator() -> IsolationProposalValidator {
    IsolationProposalValidator {
        rules: vec![
            Rc::new(IsolationProposalAllowedSourceRule {
                allowed: vec!["pending".to_string(), "rolling".to_string()],
            }) as Rc<dyn IsolationProposalRule>,
            Rc::new(IsolationProposalSourceLengthRule { max_length: 255 }),
            Rc::new(IsolationProposalRequiredCodeRule {
                message: "isolation proposal needs this field".to_string(),
            }),
            Rc::new(IsolationProposalPortRangeRule { min: 120, max: 1000 }),
            Rc::new(IsolationProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
