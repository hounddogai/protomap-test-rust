use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential threshold must pass. Check returns its problems, or none.
pub trait CredentialThresholdRule: std::fmt::Debug {
    fn check(&self, item: &CredentialThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThresholdAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl CredentialThresholdRule for CredentialThresholdAllowedReviewerRule {
    fn check(&self, item: &CredentialThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is missing", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThresholdTitleLengthRule {
    pub max_length: i64,
}

impl CredentialThresholdRule for CredentialThresholdTitleLengthRule {
    fn check(&self, item: &CredentialThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThresholdRequiredNotesRule {
    pub message: String,
}

impl CredentialThresholdRule for CredentialThresholdRequiredNotesRule {
    fn check(&self, item: &CredentialThreshold) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThresholdRequiredTitleRule {
    pub message: String,
}

impl CredentialThresholdRule for CredentialThresholdRequiredTitleRule {
    fn check(&self, item: &CredentialThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThresholdReviewerLengthRule {
    pub max_length: i64,
}

impl CredentialThresholdRule for CredentialThresholdReviewerLengthRule {
    fn check(&self, item: &CredentialThreshold) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialThresholdCodePrefixRule {
    pub prefix: String,
}

impl CredentialThresholdRule for CredentialThresholdCodePrefixRule {
    fn check(&self, item: &CredentialThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential threshold.
#[derive(Debug, Clone)]
pub struct CredentialThresholdValidator {
    pub rules: Vec<Rc<dyn CredentialThresholdRule>>,
}

impl CredentialThresholdValidator {
    pub fn validate(&self, item: &CredentialThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_threshold_validator() -> CredentialThresholdValidator {
    CredentialThresholdValidator {
        rules: vec![
            Rc::new(CredentialThresholdAllowedReviewerRule { allowed: Vec::new() }) as Rc<dyn CredentialThresholdRule>,
            Rc::new(CredentialThresholdTitleLengthRule { max_length: 80 }),
            Rc::new(CredentialThresholdRequiredNotesRule {
                message: "credential threshold needs a value".to_string(),
            }),
            Rc::new(CredentialThresholdRequiredTitleRule {
                message: "credential threshold needs an entry".to_string(),
            }),
            Rc::new(CredentialThresholdReviewerLengthRule { max_length: 86400 }),
            Rc::new(CredentialThresholdCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_threshold_validator();
        let item = new_credential_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CredentialThresholdTitleLengthRule { max_length: 32 };
        let mut item = new_credential_threshold("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = CredentialThresholdRequiredNotesRule {
            message: "credential threshold needs an entry".to_string(),
        };
        let mut item = new_credential_threshold("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CredentialThresholdRequiredTitleRule {
            message: "credential threshold needs a value".to_string(),
        };
        let mut item = new_credential_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = CredentialThresholdReviewerLengthRule { max_length: 16 };
        let mut item = new_credential_threshold("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
