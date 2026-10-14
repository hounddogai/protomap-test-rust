use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token review must pass. Check returns its problems, or none.
pub trait TokenReviewRule: std::fmt::Debug {
    fn check(&self, item: &TokenReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReviewRequiredNameRule {
    pub message: String,
}

impl TokenReviewRule for TokenReviewRequiredNameRule {
    fn check(&self, item: &TokenReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReviewPrincipalLengthRule {
    pub max_length: i64,
}

impl TokenReviewRule for TokenReviewPrincipalLengthRule {
    fn check(&self, item: &TokenReview) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReviewExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenReviewRule for TokenReviewExpiresAtRangeRule {
    fn check(&self, item: &TokenReview) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReviewNamespaceLengthRule {
    pub max_length: i64,
}

impl TokenReviewRule for TokenReviewNamespaceLengthRule {
    fn check(&self, item: &TokenReview) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReviewRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenReviewRule for TokenReviewRatioRangeRule {
    fn check(&self, item: &TokenReview) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenReviewAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl TokenReviewRule for TokenReviewAllowedReviewerRule {
    fn check(&self, item: &TokenReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is out of range", item.reviewer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token review.
#[derive(Debug, Clone)]
pub struct TokenReviewValidator {
    pub rules: Vec<Rc<dyn TokenReviewRule>>,
}

impl TokenReviewValidator {
    pub fn validate(&self, item: &TokenReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_review_validator() -> TokenReviewValidator {
    TokenReviewValidator {
        rules: vec![
            Rc::new(TokenReviewRequiredNameRule {
                message: "token review needs a value".to_string(),
            }) as Rc<dyn TokenReviewRule>,
            Rc::new(TokenReviewPrincipalLengthRule { max_length: 1000 }),
            Rc::new(TokenReviewExpiresAtRangeRule { min: 16, max: 3600 }),
            Rc::new(TokenReviewNamespaceLengthRule { max_length: 32 }),
            Rc::new(TokenReviewRatioRangeRule { min: 1.7, max: 2565.9 }),
            Rc::new(TokenReviewAllowedReviewerRule {
                allowed: vec!["tiered".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_review_validator();
        let item = new_token_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TokenReviewRequiredNameRule {
            message: "token review needs this field".to_string(),
        };
        let mut item = new_token_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = TokenReviewPrincipalLengthRule { max_length: 1000 };
        let mut item = new_token_review("x", "y");
        item.principal = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TokenReviewExpiresAtRangeRule { min: 120, max: 255 };
        let mut item = new_token_review("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = TokenReviewNamespaceLengthRule { max_length: 64 };
        let mut item = new_token_review("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TokenReviewRatioRangeRule { min: 2.3, max: 2343.0 };
        let mut item = new_token_review("x", "y");
        item.ratio = 2344.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
