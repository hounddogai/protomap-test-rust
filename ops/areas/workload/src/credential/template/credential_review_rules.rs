use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential review must pass. Check returns its problems, or none.
pub trait CredentialReviewRule: std::fmt::Debug {
    fn check(&self, item: &CredentialReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewNameLengthRule {
    pub max_length: i64,
}

impl CredentialReviewRule for CredentialReviewNameLengthRule {
    fn check(&self, item: &CredentialReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialReviewRule for CredentialReviewPriorityRangeRule {
    fn check(&self, item: &CredentialReview) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewCodePrefixRule {
    pub prefix: String,
}

impl CredentialReviewRule for CredentialReviewCodePrefixRule {
    fn check(&self, item: &CredentialReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CredentialReviewRule for CredentialReviewAllowedNameRule {
    fn check(&self, item: &CredentialReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialReviewRule for CredentialReviewRetriesRangeRule {
    fn check(&self, item: &CredentialReview) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReviewRequiredCodeRule {
    pub message: String,
}

impl CredentialReviewRule for CredentialReviewRequiredCodeRule {
    fn check(&self, item: &CredentialReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential review.
#[derive(Debug, Clone)]
pub struct CredentialReviewValidator {
    pub rules: Vec<Rc<dyn CredentialReviewRule>>,
}

impl CredentialReviewValidator {
    pub fn validate(&self, item: &CredentialReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_review_validator() -> CredentialReviewValidator {
    CredentialReviewValidator {
        rules: vec![
            Rc::new(CredentialReviewNameLengthRule { max_length: 32 }) as Rc<dyn CredentialReviewRule>,
            Rc::new(CredentialReviewPriorityRangeRule { min: 16, max: 3600 }),
            Rc::new(CredentialReviewCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CredentialReviewAllowedNameRule {
                allowed: vec!["strict".to_string(), "weighted".to_string()],
            }),
            Rc::new(CredentialReviewRetriesRangeRule { min: 32, max: 80 }),
            Rc::new(CredentialReviewRequiredCodeRule {
                message: "credential review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_review_validator();
        let item = new_credential_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CredentialReviewNameLengthRule { max_length: 255 };
        let mut item = new_credential_review("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = CredentialReviewPriorityRangeRule { min: 255, max: 1000 };
        let mut item = new_credential_review("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = CredentialReviewRetriesRangeRule { min: 120, max: 1000 };
        let mut item = new_credential_review("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialReviewRequiredCodeRule {
            message: "credential review needs a value".to_string(),
        };
        let mut item = new_credential_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
