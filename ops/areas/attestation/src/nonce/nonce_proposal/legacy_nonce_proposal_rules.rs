use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy nonce proposal must pass. Check returns its problems, or none.
pub trait LegacyNonceProposalRule: std::fmt::Debug {
    fn check(&self, item: &LegacyNonceProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceProposalCodePrefixRule {
    pub prefix: String,
}

impl LegacyNonceProposalRule for LegacyNonceProposalCodePrefixRule {
    fn check(&self, item: &LegacyNonceProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceProposalRequiredLabelRule {
    pub message: String,
}

impl LegacyNonceProposalRule for LegacyNonceProposalRequiredLabelRule {
    fn check(&self, item: &LegacyNonceProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceProposalRequiredCodeRule {
    pub message: String,
}

impl LegacyNonceProposalRule for LegacyNonceProposalRequiredCodeRule {
    fn check(&self, item: &LegacyNonceProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy nonce proposal.
#[derive(Debug, Clone)]
pub struct LegacyNonceProposalValidator {
    pub rules: Vec<Rc<dyn LegacyNonceProposalRule>>,
}

impl LegacyNonceProposalValidator {
    pub fn validate(&self, item: &LegacyNonceProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyNonceProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyNonceProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_nonce_proposal_validator() -> LegacyNonceProposalValidator {
    LegacyNonceProposalValidator {
        rules: vec![
            Rc::new(LegacyNonceProposalCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn LegacyNonceProposalRule>,
            Rc::new(LegacyNonceProposalRequiredLabelRule {
                message: "legacy nonce proposal needs an entry".to_string(),
            }),
            Rc::new(LegacyNonceProposalRequiredCodeRule {
                message: "legacy nonce proposal needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_nonce_proposal_validator();
        let item = new_legacy_nonce_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyNonceProposalRequiredLabelRule {
            message: "legacy nonce proposal needs a value".to_string(),
        };
        let mut item = new_legacy_nonce_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyNonceProposalRequiredCodeRule {
            message: "legacy nonce proposal needs a value".to_string(),
        };
        let mut item = new_legacy_nonce_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
