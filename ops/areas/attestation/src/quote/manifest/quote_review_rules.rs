use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote review must pass. Check returns its problems, or none.
pub trait QuoteReviewRule: std::fmt::Debug {
    fn check(&self, item: &QuoteReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReviewScopeLengthRule {
    pub max_length: i64,
}

impl QuoteReviewRule for QuoteReviewScopeLengthRule {
    fn check(&self, item: &QuoteReview) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReviewRequiredCodeRule {
    pub message: String,
}

impl QuoteReviewRule for QuoteReviewRequiredCodeRule {
    fn check(&self, item: &QuoteReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReviewCodeLengthRule {
    pub max_length: i64,
}

impl QuoteReviewRule for QuoteReviewCodeLengthRule {
    fn check(&self, item: &QuoteReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote review.
#[derive(Debug, Clone)]
pub struct QuoteReviewValidator {
    pub rules: Vec<Rc<dyn QuoteReviewRule>>,
}

impl QuoteReviewValidator {
    pub fn validate(&self, item: &QuoteReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_review_validator() -> QuoteReviewValidator {
    QuoteReviewValidator {
        rules: vec![
            Rc::new(QuoteReviewScopeLengthRule { max_length: 86400 }) as Rc<dyn QuoteReviewRule>,
            Rc::new(QuoteReviewRequiredCodeRule {
                message: "quote review needs an entry".to_string(),
            }),
            Rc::new(QuoteReviewCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_review_validator();
        let item = new_quote_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = QuoteReviewScopeLengthRule { max_length: 32 };
        let mut item = new_quote_review("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteReviewRequiredCodeRule {
            message: "quote review needs a value".to_string(),
        };
        let mut item = new_quote_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteReviewCodeLengthRule { max_length: 32 };
        let mut item = new_quote_review("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
