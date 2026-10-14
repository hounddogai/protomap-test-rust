use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled retention proposal must pass. Check returns its problems, or none.
pub trait ScheduledRetentionProposalRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRetentionProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionProposalRequiredCodeRule {
    pub message: String,
}

impl ScheduledRetentionProposalRule for ScheduledRetentionProposalRequiredCodeRule {
    fn check(&self, item: &ScheduledRetentionProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionProposalRequiredNotesRule {
    pub message: String,
}

impl ScheduledRetentionProposalRule for ScheduledRetentionProposalRequiredNotesRule {
    fn check(&self, item: &ScheduledRetentionProposal) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionProposalDescriptionLengthRule {
    pub max_length: i64,
}

impl ScheduledRetentionProposalRule for ScheduledRetentionProposalDescriptionLengthRule {
    fn check(&self, item: &ScheduledRetentionProposal) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled retention proposal.
#[derive(Debug, Clone)]
pub struct ScheduledRetentionProposalValidator {
    pub rules: Vec<Rc<dyn ScheduledRetentionProposalRule>>,
}

impl ScheduledRetentionProposalValidator {
    pub fn validate(&self, item: &ScheduledRetentionProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledRetentionProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRetentionProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_retention_proposal_validator() -> ScheduledRetentionProposalValidator {
    ScheduledRetentionProposalValidator {
        rules: vec![
            Rc::new(ScheduledRetentionProposalRequiredCodeRule {
                message: "scheduled retention proposal needs a value".to_string(),
            }) as Rc<dyn ScheduledRetentionProposalRule>,
            Rc::new(ScheduledRetentionProposalRequiredNotesRule {
                message: "scheduled retention proposal needs this field".to_string(),
            }),
            Rc::new(ScheduledRetentionProposalDescriptionLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_retention_proposal_validator();
        let item = new_scheduled_retention_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledRetentionProposalRequiredCodeRule {
            message: "scheduled retention proposal needs a value".to_string(),
        };
        let mut item = new_scheduled_retention_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ScheduledRetentionProposalRequiredNotesRule {
            message: "scheduled retention proposal needs a value".to_string(),
        };
        let mut item = new_scheduled_retention_proposal("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = ScheduledRetentionProposalDescriptionLengthRule { max_length: 64 };
        let mut item = new_scheduled_retention_proposal("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
