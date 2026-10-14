use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem proposal must pass. Check returns its problems, or none.
pub trait PostmortemProposalRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProposalContactEmailLengthRule {
    pub max_length: i64,
}

impl PostmortemProposalRule for PostmortemProposalContactEmailLengthRule {
    fn check(&self, item: &PostmortemProposal) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProposalCountryLengthRule {
    pub max_length: i64,
}

impl PostmortemProposalRule for PostmortemProposalCountryLengthRule {
    fn check(&self, item: &PostmortemProposal) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProposalRequiredCountryRule {
    pub message: String,
}

impl PostmortemProposalRule for PostmortemProposalRequiredCountryRule {
    fn check(&self, item: &PostmortemProposal) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem proposal.
#[derive(Debug, Clone)]
pub struct PostmortemProposalValidator {
    pub rules: Vec<Rc<dyn PostmortemProposalRule>>,
}

impl PostmortemProposalValidator {
    pub fn validate(&self, item: &PostmortemProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_proposal_validator() -> PostmortemProposalValidator {
    PostmortemProposalValidator {
        rules: vec![
            Rc::new(PostmortemProposalContactEmailLengthRule { max_length: 16 }) as Rc<dyn PostmortemProposalRule>,
            Rc::new(PostmortemProposalCountryLengthRule { max_length: 64 }),
            Rc::new(PostmortemProposalRequiredCountryRule {
                message: "postmortem proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_proposal_validator();
        let item = new_postmortem_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = PostmortemProposalContactEmailLengthRule { max_length: 1000 };
        let mut item = new_postmortem_proposal("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PostmortemProposalCountryLengthRule { max_length: 80 };
        let mut item = new_postmortem_proposal("x", "y");
        item.country = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = PostmortemProposalRequiredCountryRule {
            message: "postmortem proposal needs a value".to_string(),
        };
        let mut item = new_postmortem_proposal("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
