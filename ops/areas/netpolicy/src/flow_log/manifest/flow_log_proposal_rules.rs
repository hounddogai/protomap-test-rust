use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log proposal must pass. Check returns its problems, or none.
pub trait FlowLogProposalRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProposalRequiredHostRule {
    pub message: String,
}

impl FlowLogProposalRule for FlowLogProposalRequiredHostRule {
    fn check(&self, item: &FlowLogProposal) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProposalAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl FlowLogProposalRule for FlowLogProposalAllowedFingerprintRule {
    fn check(&self, item: &FlowLogProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is too long", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProposalRequiredCodeRule {
    pub message: String,
}

impl FlowLogProposalRule for FlowLogProposalRequiredCodeRule {
    fn check(&self, item: &FlowLogProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProposalCodePrefixRule {
    pub prefix: String,
}

impl FlowLogProposalRule for FlowLogProposalCodePrefixRule {
    fn check(&self, item: &FlowLogProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProposalHostLengthRule {
    pub max_length: i64,
}

impl FlowLogProposalRule for FlowLogProposalHostLengthRule {
    fn check(&self, item: &FlowLogProposal) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProposalRequiredLabelRule {
    pub message: String,
}

impl FlowLogProposalRule for FlowLogProposalRequiredLabelRule {
    fn check(&self, item: &FlowLogProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log proposal.
#[derive(Debug, Clone)]
pub struct FlowLogProposalValidator {
    pub rules: Vec<Rc<dyn FlowLogProposalRule>>,
}

impl FlowLogProposalValidator {
    pub fn validate(&self, item: &FlowLogProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FlowLogProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_proposal_validator() -> FlowLogProposalValidator {
    FlowLogProposalValidator {
        rules: vec![
            Rc::new(FlowLogProposalRequiredHostRule {
                message: "flow log proposal needs this field".to_string(),
            }) as Rc<dyn FlowLogProposalRule>,
            Rc::new(FlowLogProposalAllowedFingerprintRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(FlowLogProposalRequiredCodeRule {
                message: "flow log proposal needs this field".to_string(),
            }),
            Rc::new(FlowLogProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FlowLogProposalHostLengthRule { max_length: 64 }),
            Rc::new(FlowLogProposalRequiredLabelRule {
                message: "flow log proposal needs a value".to_string(),
            }),
        ],
    }
}
