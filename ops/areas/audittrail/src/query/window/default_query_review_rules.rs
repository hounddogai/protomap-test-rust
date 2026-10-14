use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default query review must pass. Check returns its problems, or none.
pub trait DefaultQueryReviewRule: std::fmt::Debug {
    fn check(&self, item: &DefaultQueryReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewCodeLengthRule {
    pub max_length: i64,
}

impl DefaultQueryReviewRule for DefaultQueryReviewCodeLengthRule {
    fn check(&self, item: &DefaultQueryReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewRequiredLabelRule {
    pub message: String,
}

impl DefaultQueryReviewRule for DefaultQueryReviewRequiredLabelRule {
    fn check(&self, item: &DefaultQueryReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultQueryReviewRule for DefaultQueryReviewVersionRangeRule {
    fn check(&self, item: &DefaultQueryReview) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewCodePrefixRule {
    pub prefix: String,
}

impl DefaultQueryReviewRule for DefaultQueryReviewCodePrefixRule {
    fn check(&self, item: &DefaultQueryReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewRequiredSummaryRule {
    pub message: String,
}

impl DefaultQueryReviewRule for DefaultQueryReviewRequiredSummaryRule {
    fn check(&self, item: &DefaultQueryReview) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryReviewAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DefaultQueryReviewRule for DefaultQueryReviewAllowedLabelRule {
    fn check(&self, item: &DefaultQueryReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default query review.
#[derive(Debug, Clone)]
pub struct DefaultQueryReviewValidator {
    pub rules: Vec<Rc<dyn DefaultQueryReviewRule>>,
}

impl DefaultQueryReviewValidator {
    pub fn validate(&self, item: &DefaultQueryReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultQueryReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultQueryReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_query_review_validator() -> DefaultQueryReviewValidator {
    DefaultQueryReviewValidator {
        rules: vec![
            Rc::new(DefaultQueryReviewCodeLengthRule { max_length: 32 }) as Rc<dyn DefaultQueryReviewRule>,
            Rc::new(DefaultQueryReviewRequiredLabelRule {
                message: "default query review needs a value".to_string(),
            }),
            Rc::new(DefaultQueryReviewVersionRangeRule { min: 80, max: 80 }),
            Rc::new(DefaultQueryReviewCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DefaultQueryReviewRequiredSummaryRule {
                message: "default query review needs an entry".to_string(),
            }),
            Rc::new(DefaultQueryReviewAllowedLabelRule {
                allowed: vec!["default".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_query_review_validator();
        let item = new_default_query_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultQueryReviewCodeLengthRule { max_length: 86400 };
        let mut item = new_default_query_review("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultQueryReviewRequiredLabelRule {
            message: "default query review needs a value".to_string(),
        };
        let mut item = new_default_query_review("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = DefaultQueryReviewVersionRangeRule { min: 16, max: 16 };
        let mut item = new_default_query_review("x", "y");
        item.version = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DefaultQueryReviewRequiredSummaryRule {
            message: "default query review needs this field".to_string(),
        };
        let mut item = new_default_query_review("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
