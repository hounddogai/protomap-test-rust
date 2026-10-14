use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist proposal must pass. Check returns its problems, or none.
pub trait AllowlistProposalRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProposalRequiredNamespaceRule {
    pub message: String,
}

impl AllowlistProposalRule for AllowlistProposalRequiredNamespaceRule {
    fn check(&self, item: &AllowlistProposal) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProposalRequiredCodeRule {
    pub message: String,
}

impl AllowlistProposalRule for AllowlistProposalRequiredCodeRule {
    fn check(&self, item: &AllowlistProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProposalTimezoneLengthRule {
    pub max_length: i64,
}

impl AllowlistProposalRule for AllowlistProposalTimezoneLengthRule {
    fn check(&self, item: &AllowlistProposal) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProposalCodePrefixRule {
    pub prefix: String,
}

impl AllowlistProposalRule for AllowlistProposalCodePrefixRule {
    fn check(&self, item: &AllowlistProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist proposal.
#[derive(Debug, Clone)]
pub struct AllowlistProposalValidator {
    pub rules: Vec<Rc<dyn AllowlistProposalRule>>,
}

impl AllowlistProposalValidator {
    pub fn validate(&self, item: &AllowlistProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_proposal_validator() -> AllowlistProposalValidator {
    AllowlistProposalValidator {
        rules: vec![
            Rc::new(AllowlistProposalRequiredNamespaceRule {
                message: "allowlist proposal needs a value".to_string(),
            }) as Rc<dyn AllowlistProposalRule>,
            Rc::new(AllowlistProposalRequiredCodeRule {
                message: "allowlist proposal needs an entry".to_string(),
            }),
            Rc::new(AllowlistProposalTimezoneLengthRule { max_length: 3600 }),
            Rc::new(AllowlistProposalCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
