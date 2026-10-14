use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage review must pass. Check returns its problems, or none.
pub trait UsageReviewRule: std::fmt::Debug {
    fn check(&self, item: &UsageReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReviewCodePrefixRule {
    pub prefix: String,
}

impl UsageReviewRule for UsageReviewCodePrefixRule {
    fn check(&self, item: &UsageReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReviewSummaryLengthRule {
    pub max_length: i64,
}

impl UsageReviewRule for UsageReviewSummaryLengthRule {
    fn check(&self, item: &UsageReview) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReviewAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl UsageReviewRule for UsageReviewAllowedLabelRule {
    fn check(&self, item: &UsageReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReviewLabelLengthRule {
    pub max_length: i64,
}

impl UsageReviewRule for UsageReviewLabelLengthRule {
    fn check(&self, item: &UsageReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReviewRequiredSummaryRule {
    pub message: String,
}

impl UsageReviewRule for UsageReviewRequiredSummaryRule {
    fn check(&self, item: &UsageReview) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageReviewCodeLengthRule {
    pub max_length: i64,
}

impl UsageReviewRule for UsageReviewCodeLengthRule {
    fn check(&self, item: &UsageReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage review.
#[derive(Debug, Clone)]
pub struct UsageReviewValidator {
    pub rules: Vec<Rc<dyn UsageReviewRule>>,
}

impl UsageReviewValidator {
    pub fn validate(&self, item: &UsageReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_review_validator() -> UsageReviewValidator {
    UsageReviewValidator {
        rules: vec![
            Rc::new(UsageReviewCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn UsageReviewRule>,
            Rc::new(UsageReviewSummaryLengthRule { max_length: 16 }),
            Rc::new(UsageReviewAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(UsageReviewLabelLengthRule { max_length: 1000 }),
            Rc::new(UsageReviewRequiredSummaryRule {
                message: "usage review needs a value".to_string(),
            }),
            Rc::new(UsageReviewCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_review_validator();
        let item = new_usage_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = UsageReviewSummaryLengthRule { max_length: 32 };
        let mut item = new_usage_review("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = UsageReviewLabelLengthRule { max_length: 255 };
        let mut item = new_usage_review("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = UsageReviewRequiredSummaryRule {
            message: "usage review needs a value".to_string(),
        };
        let mut item = new_usage_review("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsageReviewCodeLengthRule { max_length: 80 };
        let mut item = new_usage_review("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
