use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual identity proposal must pass. Check returns its problems, or none.
pub trait ManualIdentityProposalRule: std::fmt::Debug {
    fn check(&self, item: &ManualIdentityProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalRequiredClusterRule {
    pub message: String,
}

impl ManualIdentityProposalRule for ManualIdentityProposalRequiredClusterRule {
    fn check(&self, item: &ManualIdentityProposal) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalRequiredCodeRule {
    pub message: String,
}

impl ManualIdentityProposalRule for ManualIdentityProposalRequiredCodeRule {
    fn check(&self, item: &ManualIdentityProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalRequiredPrincipalRule {
    pub message: String,
}

impl ManualIdentityProposalRule for ManualIdentityProposalRequiredPrincipalRule {
    fn check(&self, item: &ManualIdentityProposal) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalLabelLengthRule {
    pub max_length: i64,
}

impl ManualIdentityProposalRule for ManualIdentityProposalLabelLengthRule {
    fn check(&self, item: &ManualIdentityProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalCodePrefixRule {
    pub prefix: String,
}

impl ManualIdentityProposalRule for ManualIdentityProposalCodePrefixRule {
    fn check(&self, item: &ManualIdentityProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityProposalAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl ManualIdentityProposalRule for ManualIdentityProposalAllowedScopeRule {
    fn check(&self, item: &ManualIdentityProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is below minimum", item.scope)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual identity proposal.
#[derive(Debug, Clone)]
pub struct ManualIdentityProposalValidator {
    pub rules: Vec<Rc<dyn ManualIdentityProposalRule>>,
}

impl ManualIdentityProposalValidator {
    pub fn validate(&self, item: &ManualIdentityProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualIdentityProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualIdentityProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_identity_proposal_validator() -> ManualIdentityProposalValidator {
    ManualIdentityProposalValidator {
        rules: vec![
            Rc::new(ManualIdentityProposalRequiredClusterRule {
                message: "manual identity proposal needs a value".to_string(),
            }) as Rc<dyn ManualIdentityProposalRule>,
            Rc::new(ManualIdentityProposalRequiredCodeRule {
                message: "manual identity proposal needs an entry".to_string(),
            }),
            Rc::new(ManualIdentityProposalRequiredPrincipalRule {
                message: "manual identity proposal needs an entry".to_string(),
            }),
            Rc::new(ManualIdentityProposalLabelLengthRule { max_length: 32 }),
            Rc::new(ManualIdentityProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualIdentityProposalAllowedScopeRule { allowed: Vec::new() }),
        ],
    }
}
