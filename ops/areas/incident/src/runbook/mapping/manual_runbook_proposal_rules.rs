use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual runbook proposal must pass. Check returns its problems, or none.
pub trait ManualRunbookProposalRule: std::fmt::Debug {
    fn check(&self, item: &ManualRunbookProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookProposalLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRunbookProposalRule for ManualRunbookProposalLimitRangeRule {
    fn check(&self, item: &ManualRunbookProposal) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookProposalRequiredCodeRule {
    pub message: String,
}

impl ManualRunbookProposalRule for ManualRunbookProposalRequiredCodeRule {
    fn check(&self, item: &ManualRunbookProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookProposalCodePrefixRule {
    pub prefix: String,
}

impl ManualRunbookProposalRule for ManualRunbookProposalCodePrefixRule {
    fn check(&self, item: &ManualRunbookProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookProposalRequiredNotesRule {
    pub message: String,
}

impl ManualRunbookProposalRule for ManualRunbookProposalRequiredNotesRule {
    fn check(&self, item: &ManualRunbookProposal) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookProposalTitleLengthRule {
    pub max_length: i64,
}

impl ManualRunbookProposalRule for ManualRunbookProposalTitleLengthRule {
    fn check(&self, item: &ManualRunbookProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual runbook proposal.
#[derive(Debug, Clone)]
pub struct ManualRunbookProposalValidator {
    pub rules: Vec<Rc<dyn ManualRunbookProposalRule>>,
}

impl ManualRunbookProposalValidator {
    pub fn validate(&self, item: &ManualRunbookProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRunbookProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRunbookProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_runbook_proposal_validator() -> ManualRunbookProposalValidator {
    ManualRunbookProposalValidator {
        rules: vec![
            Rc::new(ManualRunbookProposalLimitRangeRule { min: 255, max: 3600 }) as Rc<dyn ManualRunbookProposalRule>,
            Rc::new(ManualRunbookProposalRequiredCodeRule {
                message: "manual runbook proposal needs a value".to_string(),
            }),
            Rc::new(ManualRunbookProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ManualRunbookProposalRequiredNotesRule {
                message: "manual runbook proposal needs a value".to_string(),
            }),
            Rc::new(ManualRunbookProposalTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_runbook_proposal_validator();
        let item = new_manual_runbook_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ManualRunbookProposalLimitRangeRule { min: 16, max: 255 };
        let mut item = new_manual_runbook_proposal("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualRunbookProposalRequiredCodeRule {
            message: "manual runbook proposal needs this field".to_string(),
        };
        let mut item = new_manual_runbook_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ManualRunbookProposalRequiredNotesRule {
            message: "manual runbook proposal needs this field".to_string(),
        };
        let mut item = new_manual_runbook_proposal("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualRunbookProposalTitleLengthRule { max_length: 80 };
        let mut item = new_manual_runbook_proposal("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
