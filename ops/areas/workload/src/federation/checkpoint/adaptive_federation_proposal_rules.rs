use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive federation proposal must pass. Check returns its problems, or none.
pub trait AdaptiveFederationProposalRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFederationProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationProposalRequiredCodeRule {
    pub message: String,
}

impl AdaptiveFederationProposalRule for AdaptiveFederationProposalRequiredCodeRule {
    fn check(&self, item: &AdaptiveFederationProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationProposalMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveFederationProposalRule for AdaptiveFederationProposalMinimumRangeRule {
    fn check(&self, item: &AdaptiveFederationProposal) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationProposalCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveFederationProposalRule for AdaptiveFederationProposalCodePrefixRule {
    fn check(&self, item: &AdaptiveFederationProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive federation proposal.
#[derive(Debug, Clone)]
pub struct AdaptiveFederationProposalValidator {
    pub rules: Vec<Rc<dyn AdaptiveFederationProposalRule>>,
}

impl AdaptiveFederationProposalValidator {
    pub fn validate(&self, item: &AdaptiveFederationProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveFederationProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFederationProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_federation_proposal_validator() -> AdaptiveFederationProposalValidator {
    AdaptiveFederationProposalValidator {
        rules: vec![
            Rc::new(AdaptiveFederationProposalRequiredCodeRule {
                message: "adaptive federation proposal needs an entry".to_string(),
            }) as Rc<dyn AdaptiveFederationProposalRule>,
            Rc::new(AdaptiveFederationProposalMinimumRangeRule { min: 7.3, max: 2324.3 }),
            Rc::new(AdaptiveFederationProposalCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
