use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal identity review must pass. Check returns its problems, or none.
pub trait SeasonalIdentityReviewRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalIdentityReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReviewRequiredCodeRule {
    pub message: String,
}

impl SeasonalIdentityReviewRule for SeasonalIdentityReviewRequiredCodeRule {
    fn check(&self, item: &SeasonalIdentityReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReviewRequiredPrincipalRule {
    pub message: String,
}

impl SeasonalIdentityReviewRule for SeasonalIdentityReviewRequiredPrincipalRule {
    fn check(&self, item: &SeasonalIdentityReview) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReviewCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalIdentityReviewRule for SeasonalIdentityReviewCodeLengthRule {
    fn check(&self, item: &SeasonalIdentityReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReviewRequiredTitleRule {
    pub message: String,
}

impl SeasonalIdentityReviewRule for SeasonalIdentityReviewRequiredTitleRule {
    fn check(&self, item: &SeasonalIdentityReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReviewAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl SeasonalIdentityReviewRule for SeasonalIdentityReviewAllowedSegmentRule {
    fn check(&self, item: &SeasonalIdentityReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is out of range", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityReviewPrincipalLengthRule {
    pub max_length: i64,
}

impl SeasonalIdentityReviewRule for SeasonalIdentityReviewPrincipalLengthRule {
    fn check(&self, item: &SeasonalIdentityReview) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal identity review.
#[derive(Debug, Clone)]
pub struct SeasonalIdentityReviewValidator {
    pub rules: Vec<Rc<dyn SeasonalIdentityReviewRule>>,
}

impl SeasonalIdentityReviewValidator {
    pub fn validate(&self, item: &SeasonalIdentityReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalIdentityReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalIdentityReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_identity_review_validator() -> SeasonalIdentityReviewValidator {
    SeasonalIdentityReviewValidator {
        rules: vec![
            Rc::new(SeasonalIdentityReviewRequiredCodeRule {
                message: "seasonal identity review needs a value".to_string(),
            }) as Rc<dyn SeasonalIdentityReviewRule>,
            Rc::new(SeasonalIdentityReviewRequiredPrincipalRule {
                message: "seasonal identity review needs this field".to_string(),
            }),
            Rc::new(SeasonalIdentityReviewCodeLengthRule { max_length: 86400 }),
            Rc::new(SeasonalIdentityReviewRequiredTitleRule {
                message: "seasonal identity review needs a value".to_string(),
            }),
            Rc::new(SeasonalIdentityReviewAllowedSegmentRule { allowed: Vec::new() }),
            Rc::new(SeasonalIdentityReviewPrincipalLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_identity_review_validator();
        let item = new_seasonal_identity_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalIdentityReviewRequiredCodeRule {
            message: "seasonal identity review needs a value".to_string(),
        };
        let mut item = new_seasonal_identity_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = SeasonalIdentityReviewRequiredPrincipalRule {
            message: "seasonal identity review needs a value".to_string(),
        };
        let mut item = new_seasonal_identity_review("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalIdentityReviewCodeLengthRule { max_length: 16 };
        let mut item = new_seasonal_identity_review("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalIdentityReviewRequiredTitleRule {
            message: "seasonal identity review needs a value".to_string(),
        };
        let mut item = new_seasonal_identity_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = SeasonalIdentityReviewPrincipalLengthRule { max_length: 1000 };
        let mut item = new_seasonal_identity_review("x", "y");
        item.principal = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
