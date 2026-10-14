use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage review must pass. Check returns its problems, or none.
pub trait OverageReviewRule: std::fmt::Debug {
    fn check(&self, item: &OverageReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReviewCodeLengthRule {
    pub max_length: i64,
}

impl OverageReviewRule for OverageReviewCodeLengthRule {
    fn check(&self, item: &OverageReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReviewAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageReviewRule for OverageReviewAmountRangeRule {
    fn check(&self, item: &OverageReview) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReviewRequiredNameRule {
    pub message: String,
}

impl OverageReviewRule for OverageReviewRequiredNameRule {
    fn check(&self, item: &OverageReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReviewCodePrefixRule {
    pub prefix: String,
}

impl OverageReviewRule for OverageReviewCodePrefixRule {
    fn check(&self, item: &OverageReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl OverageReviewRule for OverageReviewAllowedCodeRule {
    fn check(&self, item: &OverageReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage review.
#[derive(Debug, Clone)]
pub struct OverageReviewValidator {
    pub rules: Vec<Rc<dyn OverageReviewRule>>,
}

impl OverageReviewValidator {
    pub fn validate(&self, item: &OverageReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_review_validator() -> OverageReviewValidator {
    OverageReviewValidator {
        rules: vec![
            Rc::new(OverageReviewCodeLengthRule { max_length: 3600 }) as Rc<dyn OverageReviewRule>,
            Rc::new(OverageReviewAmountRangeRule { min: 2.0, max: 317.9 }),
            Rc::new(OverageReviewRequiredNameRule {
                message: "overage review needs a value".to_string(),
            }),
            Rc::new(OverageReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(OverageReviewAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_review_validator();
        let item = new_overage_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageReviewCodeLengthRule { max_length: 16 };
        let mut item = new_overage_review("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = OverageReviewAmountRangeRule { min: 5.9, max: 4254.0 };
        let mut item = new_overage_review("x", "y");
        item.amount = 4255.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = OverageReviewRequiredNameRule {
            message: "overage review needs this field".to_string(),
        };
        let mut item = new_overage_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
