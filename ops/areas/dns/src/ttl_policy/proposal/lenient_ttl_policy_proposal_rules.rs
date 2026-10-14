use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient ttl policy proposal must pass. Check returns its problems, or none.
pub trait LenientTtlPolicyProposalRule: std::fmt::Debug {
    fn check(&self, item: &LenientTtlPolicyProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyProposalCodePrefixRule {
    pub prefix: String,
}

impl LenientTtlPolicyProposalRule for LenientTtlPolicyProposalCodePrefixRule {
    fn check(&self, item: &LenientTtlPolicyProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyProposalNamespaceLengthRule {
    pub max_length: i64,
}

impl LenientTtlPolicyProposalRule for LenientTtlPolicyProposalNamespaceLengthRule {
    fn check(&self, item: &LenientTtlPolicyProposal) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyProposalRequiredLabelRule {
    pub message: String,
}

impl LenientTtlPolicyProposalRule for LenientTtlPolicyProposalRequiredLabelRule {
    fn check(&self, item: &LenientTtlPolicyProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyProposalLabelLengthRule {
    pub max_length: i64,
}

impl LenientTtlPolicyProposalRule for LenientTtlPolicyProposalLabelLengthRule {
    fn check(&self, item: &LenientTtlPolicyProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient ttl policy proposal.
#[derive(Debug, Clone)]
pub struct LenientTtlPolicyProposalValidator {
    pub rules: Vec<Rc<dyn LenientTtlPolicyProposalRule>>,
}

impl LenientTtlPolicyProposalValidator {
    pub fn validate(&self, item: &LenientTtlPolicyProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTtlPolicyProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTtlPolicyProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_ttl_policy_proposal_validator() -> LenientTtlPolicyProposalValidator {
    LenientTtlPolicyProposalValidator {
        rules: vec![
            Rc::new(LenientTtlPolicyProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn LenientTtlPolicyProposalRule>,
            Rc::new(LenientTtlPolicyProposalNamespaceLengthRule { max_length: 255 }),
            Rc::new(LenientTtlPolicyProposalRequiredLabelRule {
                message: "lenient ttl policy proposal needs this field".to_string(),
            }),
            Rc::new(LenientTtlPolicyProposalLabelLengthRule { max_length: 16 }),
        ],
    }
}
