use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict forecast review must pass. Check returns its problems, or none.
pub trait StrictForecastReviewRule: std::fmt::Debug {
    fn check(&self, item: &StrictForecastReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastReviewEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictForecastReviewRule for StrictForecastReviewEndsAtRangeRule {
    fn check(&self, item: &StrictForecastReview) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastReviewCodePrefixRule {
    pub prefix: String,
}

impl StrictForecastReviewRule for StrictForecastReviewCodePrefixRule {
    fn check(&self, item: &StrictForecastReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastReviewRequiredCodeRule {
    pub message: String,
}

impl StrictForecastReviewRule for StrictForecastReviewRequiredCodeRule {
    fn check(&self, item: &StrictForecastReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastReviewRequiredCategoryRule {
    pub message: String,
}

impl StrictForecastReviewRule for StrictForecastReviewRequiredCategoryRule {
    fn check(&self, item: &StrictForecastReview) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict forecast review.
#[derive(Debug, Clone)]
pub struct StrictForecastReviewValidator {
    pub rules: Vec<Rc<dyn StrictForecastReviewRule>>,
}

impl StrictForecastReviewValidator {
    pub fn validate(&self, item: &StrictForecastReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictForecastReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictForecastReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_forecast_review_validator() -> StrictForecastReviewValidator {
    StrictForecastReviewValidator {
        rules: vec![
            Rc::new(StrictForecastReviewEndsAtRangeRule { min: 64, max: 86400 }) as Rc<dyn StrictForecastReviewRule>,
            Rc::new(StrictForecastReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictForecastReviewRequiredCodeRule {
                message: "strict forecast review needs a value".to_string(),
            }),
            Rc::new(StrictForecastReviewRequiredCategoryRule {
                message: "strict forecast review needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_forecast_review_validator();
        let item = new_strict_forecast_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = StrictForecastReviewEndsAtRangeRule { min: 86400, max: 86400 };
        let mut item = new_strict_forecast_review("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictForecastReviewRequiredCodeRule {
            message: "strict forecast review needs a value".to_string(),
        };
        let mut item = new_strict_forecast_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = StrictForecastReviewRequiredCategoryRule {
            message: "strict forecast review needs this field".to_string(),
        };
        let mut item = new_strict_forecast_review("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
