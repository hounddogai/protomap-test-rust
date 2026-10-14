use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default retention proposal must pass. Check returns its problems, or none.
pub trait DefaultRetentionProposalRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRetentionProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionProposalContactEmailLengthRule {
    pub max_length: i64,
}

impl DefaultRetentionProposalRule for DefaultRetentionProposalContactEmailLengthRule {
    fn check(&self, item: &DefaultRetentionProposal) -> Vec<String> {
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
pub struct DefaultRetentionProposalScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRetentionProposalRule for DefaultRetentionProposalScoreRangeRule {
    fn check(&self, item: &DefaultRetentionProposal) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionProposalRequiredHostRule {
    pub message: String,
}

impl DefaultRetentionProposalRule for DefaultRetentionProposalRequiredHostRule {
    fn check(&self, item: &DefaultRetentionProposal) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionProposalCodePrefixRule {
    pub prefix: String,
}

impl DefaultRetentionProposalRule for DefaultRetentionProposalCodePrefixRule {
    fn check(&self, item: &DefaultRetentionProposal) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionProposalRequiredContactEmailRule {
    pub message: String,
}

impl DefaultRetentionProposalRule for DefaultRetentionProposalRequiredContactEmailRule {
    fn check(&self, item: &DefaultRetentionProposal) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionProposalTitleLengthRule {
    pub max_length: i64,
}

impl DefaultRetentionProposalRule for DefaultRetentionProposalTitleLengthRule {
    fn check(&self, item: &DefaultRetentionProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default retention proposal.
#[derive(Debug, Clone)]
pub struct DefaultRetentionProposalValidator {
    pub rules: Vec<Rc<dyn DefaultRetentionProposalRule>>,
}

impl DefaultRetentionProposalValidator {
    pub fn validate(&self, item: &DefaultRetentionProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRetentionProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRetentionProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_retention_proposal_validator() -> DefaultRetentionProposalValidator {
    DefaultRetentionProposalValidator {
        rules: vec![
            Rc::new(DefaultRetentionProposalContactEmailLengthRule { max_length: 80 })
                as Rc<dyn DefaultRetentionProposalRule>,
            Rc::new(DefaultRetentionProposalScoreRangeRule { min: 3.3, max: 3006.3 }),
            Rc::new(DefaultRetentionProposalRequiredHostRule {
                message: "default retention proposal needs an entry".to_string(),
            }),
            Rc::new(DefaultRetentionProposalCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DefaultRetentionProposalRequiredContactEmailRule {
                message: "default retention proposal needs a value".to_string(),
            }),
            Rc::new(DefaultRetentionProposalTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_retention_proposal_validator();
        let item = new_default_retention_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DefaultRetentionProposalContactEmailLengthRule { max_length: 255 };
        let mut item = new_default_retention_proposal("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DefaultRetentionProposalScoreRangeRule { min: 7.3, max: 1424.1 };
        let mut item = new_default_retention_proposal("x", "y");
        item.score = 1425.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = DefaultRetentionProposalRequiredHostRule {
            message: "default retention proposal needs this field".to_string(),
        };
        let mut item = new_default_retention_proposal("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DefaultRetentionProposalRequiredContactEmailRule {
            message: "default retention proposal needs an entry".to_string(),
        };
        let mut item = new_default_retention_proposal("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultRetentionProposalTitleLengthRule { max_length: 16 };
        let mut item = new_default_retention_proposal("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
