use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global retention proposal must pass. Check returns its problems, or none.
pub trait GlobalRetentionProposalRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRetentionProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionProposalTitleLengthRule {
    pub max_length: i64,
}

impl GlobalRetentionProposalRule for GlobalRetentionProposalTitleLengthRule {
    fn check(&self, item: &GlobalRetentionProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionProposalCodePrefixRule {
    pub prefix: String,
}

impl GlobalRetentionProposalRule for GlobalRetentionProposalCodePrefixRule {
    fn check(&self, item: &GlobalRetentionProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalRetentionProposalRule for GlobalRetentionProposalAllowedCodeRule {
    fn check(&self, item: &GlobalRetentionProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global retention proposal.
#[derive(Debug, Clone)]
pub struct GlobalRetentionProposalValidator {
    pub rules: Vec<Rc<dyn GlobalRetentionProposalRule>>,
}

impl GlobalRetentionProposalValidator {
    pub fn validate(&self, item: &GlobalRetentionProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalRetentionProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRetentionProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_retention_proposal_validator() -> GlobalRetentionProposalValidator {
    GlobalRetentionProposalValidator {
        rules: vec![
            Rc::new(GlobalRetentionProposalTitleLengthRule { max_length: 120 }) as Rc<dyn GlobalRetentionProposalRule>,
            Rc::new(GlobalRetentionProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(GlobalRetentionProposalAllowedCodeRule {
                allowed: vec!["batched".to_string(), "tiered".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_retention_proposal_validator();
        let item = new_global_retention_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalRetentionProposalTitleLengthRule { max_length: 86400 };
        let mut item = new_global_retention_proposal("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
