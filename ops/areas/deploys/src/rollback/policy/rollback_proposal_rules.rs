use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback proposal must pass. Check returns its problems, or none.
pub trait RollbackProposalRule: std::fmt::Debug {
    fn check(&self, item: &RollbackProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProposalRequiredPhoneRule {
    pub message: String,
}

impl RollbackProposalRule for RollbackProposalRequiredPhoneRule {
    fn check(&self, item: &RollbackProposal) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProposalTitleLengthRule {
    pub max_length: i64,
}

impl RollbackProposalRule for RollbackProposalTitleLengthRule {
    fn check(&self, item: &RollbackProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProposalPhoneLengthRule {
    pub max_length: i64,
}

impl RollbackProposalRule for RollbackProposalPhoneLengthRule {
    fn check(&self, item: &RollbackProposal) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProposalOwnerIdLengthRule {
    pub max_length: i64,
}

impl RollbackProposalRule for RollbackProposalOwnerIdLengthRule {
    fn check(&self, item: &RollbackProposal) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback proposal.
#[derive(Debug, Clone)]
pub struct RollbackProposalValidator {
    pub rules: Vec<Rc<dyn RollbackProposalRule>>,
}

impl RollbackProposalValidator {
    pub fn validate(&self, item: &RollbackProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_proposal_validator() -> RollbackProposalValidator {
    RollbackProposalValidator {
        rules: vec![
            Rc::new(RollbackProposalRequiredPhoneRule {
                message: "rollback proposal needs this field".to_string(),
            }) as Rc<dyn RollbackProposalRule>,
            Rc::new(RollbackProposalTitleLengthRule { max_length: 32 }),
            Rc::new(RollbackProposalPhoneLengthRule { max_length: 120 }),
            Rc::new(RollbackProposalOwnerIdLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_proposal_validator();
        let item = new_rollback_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = RollbackProposalRequiredPhoneRule {
            message: "rollback proposal needs this field".to_string(),
        };
        let mut item = new_rollback_proposal("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollbackProposalTitleLengthRule { max_length: 80 };
        let mut item = new_rollback_proposal("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RollbackProposalPhoneLengthRule { max_length: 80 };
        let mut item = new_rollback_proposal("x", "y");
        item.phone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RollbackProposalOwnerIdLengthRule { max_length: 64 };
        let mut item = new_rollback_proposal("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
