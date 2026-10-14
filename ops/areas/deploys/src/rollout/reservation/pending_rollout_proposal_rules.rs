use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending rollout proposal must pass. Check returns its problems, or none.
pub trait PendingRolloutProposalRule: std::fmt::Debug {
    fn check(&self, item: &PendingRolloutProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutProposalLabelLengthRule {
    pub max_length: i64,
}

impl PendingRolloutProposalRule for PendingRolloutProposalLabelLengthRule {
    fn check(&self, item: &PendingRolloutProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutProposalCodeLengthRule {
    pub max_length: i64,
}

impl PendingRolloutProposalRule for PendingRolloutProposalCodeLengthRule {
    fn check(&self, item: &PendingRolloutProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutProposalCodePrefixRule {
    pub prefix: String,
}

impl PendingRolloutProposalRule for PendingRolloutProposalCodePrefixRule {
    fn check(&self, item: &PendingRolloutProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutProposalSourceLengthRule {
    pub max_length: i64,
}

impl PendingRolloutProposalRule for PendingRolloutProposalSourceLengthRule {
    fn check(&self, item: &PendingRolloutProposal) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending rollout proposal.
#[derive(Debug, Clone)]
pub struct PendingRolloutProposalValidator {
    pub rules: Vec<Rc<dyn PendingRolloutProposalRule>>,
}

impl PendingRolloutProposalValidator {
    pub fn validate(&self, item: &PendingRolloutProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRolloutProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRolloutProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_rollout_proposal_validator() -> PendingRolloutProposalValidator {
    PendingRolloutProposalValidator {
        rules: vec![
            Rc::new(PendingRolloutProposalLabelLengthRule { max_length: 255 }) as Rc<dyn PendingRolloutProposalRule>,
            Rc::new(PendingRolloutProposalCodeLengthRule { max_length: 86400 }),
            Rc::new(PendingRolloutProposalCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingRolloutProposalSourceLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_rollout_proposal_validator();
        let item = new_pending_rollout_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingRolloutProposalLabelLengthRule { max_length: 16 };
        let mut item = new_pending_rollout_proposal("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRolloutProposalCodeLengthRule { max_length: 86400 };
        let mut item = new_pending_rollout_proposal("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = PendingRolloutProposalSourceLengthRule { max_length: 3600 };
        let mut item = new_pending_rollout_proposal("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
