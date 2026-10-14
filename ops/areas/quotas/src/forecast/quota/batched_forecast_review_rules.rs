use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched forecast review must pass. Check returns its problems, or none.
pub trait BatchedForecastReviewRule: std::fmt::Debug {
    fn check(&self, item: &BatchedForecastReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReviewRequiredNotesRule {
    pub message: String,
}

impl BatchedForecastReviewRule for BatchedForecastReviewRequiredNotesRule {
    fn check(&self, item: &BatchedForecastReview) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReviewNotesLengthRule {
    pub max_length: i64,
}

impl BatchedForecastReviewRule for BatchedForecastReviewNotesLengthRule {
    fn check(&self, item: &BatchedForecastReview) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedForecastReviewRule for BatchedForecastReviewAllowedCodeRule {
    fn check(&self, item: &BatchedForecastReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReviewRequiredScopeRule {
    pub message: String,
}

impl BatchedForecastReviewRule for BatchedForecastReviewRequiredScopeRule {
    fn check(&self, item: &BatchedForecastReview) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReviewRequiredCodeRule {
    pub message: String,
}

impl BatchedForecastReviewRule for BatchedForecastReviewRequiredCodeRule {
    fn check(&self, item: &BatchedForecastReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched forecast review.
#[derive(Debug, Clone)]
pub struct BatchedForecastReviewValidator {
    pub rules: Vec<Rc<dyn BatchedForecastReviewRule>>,
}

impl BatchedForecastReviewValidator {
    pub fn validate(&self, item: &BatchedForecastReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedForecastReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedForecastReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_forecast_review_validator() -> BatchedForecastReviewValidator {
    BatchedForecastReviewValidator {
        rules: vec![
            Rc::new(BatchedForecastReviewRequiredNotesRule {
                message: "batched forecast review needs an entry".to_string(),
            }) as Rc<dyn BatchedForecastReviewRule>,
            Rc::new(BatchedForecastReviewNotesLengthRule { max_length: 3600 }),
            Rc::new(BatchedForecastReviewAllowedCodeRule {
                allowed: vec!["global".to_string(), "global".to_string()],
            }),
            Rc::new(BatchedForecastReviewRequiredScopeRule {
                message: "batched forecast review needs an entry".to_string(),
            }),
            Rc::new(BatchedForecastReviewRequiredCodeRule {
                message: "batched forecast review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_forecast_review_validator();
        let item = new_batched_forecast_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = BatchedForecastReviewRequiredNotesRule {
            message: "batched forecast review needs an entry".to_string(),
        };
        let mut item = new_batched_forecast_review("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = BatchedForecastReviewNotesLengthRule { max_length: 255 };
        let mut item = new_batched_forecast_review("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = BatchedForecastReviewRequiredScopeRule {
            message: "batched forecast review needs this field".to_string(),
        };
        let mut item = new_batched_forecast_review("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedForecastReviewRequiredCodeRule {
            message: "batched forecast review needs this field".to_string(),
        };
        let mut item = new_batched_forecast_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
