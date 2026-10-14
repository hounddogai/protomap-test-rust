use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global limit proposal must pass. Check returns its problems, or none.
pub trait GlobalLimitProposalRule: std::fmt::Debug {
    fn check(&self, item: &GlobalLimitProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitProposalCategoryLengthRule {
    pub max_length: i64,
}

impl GlobalLimitProposalRule for GlobalLimitProposalCategoryLengthRule {
    fn check(&self, item: &GlobalLimitProposal) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitProposalRequiredCodeRule {
    pub message: String,
}

impl GlobalLimitProposalRule for GlobalLimitProposalRequiredCodeRule {
    fn check(&self, item: &GlobalLimitProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitProposalRequiredNameRule {
    pub message: String,
}

impl GlobalLimitProposalRule for GlobalLimitProposalRequiredNameRule {
    fn check(&self, item: &GlobalLimitProposal) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitProposalNameLengthRule {
    pub max_length: i64,
}

impl GlobalLimitProposalRule for GlobalLimitProposalNameLengthRule {
    fn check(&self, item: &GlobalLimitProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitProposalCodePrefixRule {
    pub prefix: String,
}

impl GlobalLimitProposalRule for GlobalLimitProposalCodePrefixRule {
    fn check(&self, item: &GlobalLimitProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global limit proposal.
#[derive(Debug, Clone)]
pub struct GlobalLimitProposalValidator {
    pub rules: Vec<Rc<dyn GlobalLimitProposalRule>>,
}

impl GlobalLimitProposalValidator {
    pub fn validate(&self, item: &GlobalLimitProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalLimitProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalLimitProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_limit_proposal_validator() -> GlobalLimitProposalValidator {
    GlobalLimitProposalValidator {
        rules: vec![
            Rc::new(GlobalLimitProposalCategoryLengthRule { max_length: 64 }) as Rc<dyn GlobalLimitProposalRule>,
            Rc::new(GlobalLimitProposalRequiredCodeRule {
                message: "global limit proposal needs an entry".to_string(),
            }),
            Rc::new(GlobalLimitProposalRequiredNameRule {
                message: "global limit proposal needs an entry".to_string(),
            }),
            Rc::new(GlobalLimitProposalNameLengthRule { max_length: 3600 }),
            Rc::new(GlobalLimitProposalCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
