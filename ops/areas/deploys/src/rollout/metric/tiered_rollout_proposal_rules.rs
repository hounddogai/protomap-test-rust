use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered rollout proposal must pass. Check returns its problems, or none.
pub trait TieredRolloutProposalRule: std::fmt::Debug {
    fn check(&self, item: &TieredRolloutProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutProposalAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl TieredRolloutProposalRule for TieredRolloutProposalAllowedKeyIdRule {
    fn check(&self, item: &TieredRolloutProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is duplicated", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutProposalRequiredLabelRule {
    pub message: String,
}

impl TieredRolloutProposalRule for TieredRolloutProposalRequiredLabelRule {
    fn check(&self, item: &TieredRolloutProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutProposalLabelLengthRule {
    pub max_length: i64,
}

impl TieredRolloutProposalRule for TieredRolloutProposalLabelLengthRule {
    fn check(&self, item: &TieredRolloutProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutProposalCodeLengthRule {
    pub max_length: i64,
}

impl TieredRolloutProposalRule for TieredRolloutProposalCodeLengthRule {
    fn check(&self, item: &TieredRolloutProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered rollout proposal.
#[derive(Debug, Clone)]
pub struct TieredRolloutProposalValidator {
    pub rules: Vec<Rc<dyn TieredRolloutProposalRule>>,
}

impl TieredRolloutProposalValidator {
    pub fn validate(&self, item: &TieredRolloutProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredRolloutProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRolloutProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_rollout_proposal_validator() -> TieredRolloutProposalValidator {
    TieredRolloutProposalValidator {
        rules: vec![
            Rc::new(TieredRolloutProposalAllowedKeyIdRule { allowed: Vec::new() }) as Rc<dyn TieredRolloutProposalRule>,
            Rc::new(TieredRolloutProposalRequiredLabelRule {
                message: "tiered rollout proposal needs a value".to_string(),
            }),
            Rc::new(TieredRolloutProposalLabelLengthRule { max_length: 32 }),
            Rc::new(TieredRolloutProposalCodeLengthRule { max_length: 80 }),
        ],
    }
}
