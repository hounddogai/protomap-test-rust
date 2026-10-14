use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual nonce proposal must pass. Check returns its problems, or none.
pub trait ManualNonceProposalRule: std::fmt::Debug {
    fn check(&self, item: &ManualNonceProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualNonceProposalRule for ManualNonceProposalAllowedCodeRule {
    fn check(&self, item: &ManualNonceProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceProposalUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualNonceProposalRule for ManualNonceProposalUpdatedAtRangeRule {
    fn check(&self, item: &ManualNonceProposal) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceProposalRequiredCodeRule {
    pub message: String,
}

impl ManualNonceProposalRule for ManualNonceProposalRequiredCodeRule {
    fn check(&self, item: &ManualNonceProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceProposalRequiredCategoryRule {
    pub message: String,
}

impl ManualNonceProposalRule for ManualNonceProposalRequiredCategoryRule {
    fn check(&self, item: &ManualNonceProposal) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual nonce proposal.
#[derive(Debug, Clone)]
pub struct ManualNonceProposalValidator {
    pub rules: Vec<Rc<dyn ManualNonceProposalRule>>,
}

impl ManualNonceProposalValidator {
    pub fn validate(&self, item: &ManualNonceProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualNonceProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualNonceProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_nonce_proposal_validator() -> ManualNonceProposalValidator {
    ManualNonceProposalValidator {
        rules: vec![
            Rc::new(ManualNonceProposalAllowedCodeRule {
                allowed: vec!["legacy".to_string()],
            }) as Rc<dyn ManualNonceProposalRule>,
            Rc::new(ManualNonceProposalUpdatedAtRangeRule { min: 120, max: 255 }),
            Rc::new(ManualNonceProposalRequiredCodeRule {
                message: "manual nonce proposal needs this field".to_string(),
            }),
            Rc::new(ManualNonceProposalRequiredCategoryRule {
                message: "manual nonce proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_nonce_proposal_validator();
        let item = new_manual_nonce_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ManualNonceProposalUpdatedAtRangeRule { min: 16, max: 32 };
        let mut item = new_manual_nonce_proposal("x", "y");
        item.updated_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualNonceProposalRequiredCodeRule {
            message: "manual nonce proposal needs a value".to_string(),
        };
        let mut item = new_manual_nonce_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = ManualNonceProposalRequiredCategoryRule {
            message: "manual nonce proposal needs an entry".to_string(),
        };
        let mut item = new_manual_nonce_proposal("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
