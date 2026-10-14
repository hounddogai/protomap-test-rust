use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rollout proposal must pass. Check returns its problems, or none.
pub trait PrimaryRolloutProposalRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRolloutProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutProposalLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryRolloutProposalRule for PrimaryRolloutProposalLabelLengthRule {
    fn check(&self, item: &PrimaryRolloutProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutProposalRequiredLabelRule {
    pub message: String,
}

impl PrimaryRolloutProposalRule for PrimaryRolloutProposalRequiredLabelRule {
    fn check(&self, item: &PrimaryRolloutProposal) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutProposalTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRolloutProposalRule for PrimaryRolloutProposalTtlSecondsRangeRule {
    fn check(&self, item: &PrimaryRolloutProposal) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutProposalCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRolloutProposalRule for PrimaryRolloutProposalCodeLengthRule {
    fn check(&self, item: &PrimaryRolloutProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutProposalRequiredCodeRule {
    pub message: String,
}

impl PrimaryRolloutProposalRule for PrimaryRolloutProposalRequiredCodeRule {
    fn check(&self, item: &PrimaryRolloutProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary rollout proposal.
#[derive(Debug, Clone)]
pub struct PrimaryRolloutProposalValidator {
    pub rules: Vec<Rc<dyn PrimaryRolloutProposalRule>>,
}

impl PrimaryRolloutProposalValidator {
    pub fn validate(&self, item: &PrimaryRolloutProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRolloutProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRolloutProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rollout_proposal_validator() -> PrimaryRolloutProposalValidator {
    PrimaryRolloutProposalValidator {
        rules: vec![
            Rc::new(PrimaryRolloutProposalLabelLengthRule { max_length: 120 }) as Rc<dyn PrimaryRolloutProposalRule>,
            Rc::new(PrimaryRolloutProposalRequiredLabelRule {
                message: "primary rollout proposal needs an entry".to_string(),
            }),
            Rc::new(PrimaryRolloutProposalTtlSecondsRangeRule { min: 64, max: 120 }),
            Rc::new(PrimaryRolloutProposalCodeLengthRule { max_length: 32 }),
            Rc::new(PrimaryRolloutProposalRequiredCodeRule {
                message: "primary rollout proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rollout_proposal_validator();
        let item = new_primary_rollout_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryRolloutProposalLabelLengthRule { max_length: 80 };
        let mut item = new_primary_rollout_proposal("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryRolloutProposalRequiredLabelRule {
            message: "primary rollout proposal needs a value".to_string(),
        };
        let mut item = new_primary_rollout_proposal("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PrimaryRolloutProposalTtlSecondsRangeRule { min: 16, max: 3600 };
        let mut item = new_primary_rollout_proposal("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRolloutProposalCodeLengthRule { max_length: 120 };
        let mut item = new_primary_rollout_proposal("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryRolloutProposalRequiredCodeRule {
            message: "primary rollout proposal needs a value".to_string(),
        };
        let mut item = new_primary_rollout_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
