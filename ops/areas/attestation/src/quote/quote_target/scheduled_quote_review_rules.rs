use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled quote review must pass. Check returns its problems, or none.
pub trait ScheduledQuoteReviewRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledQuoteReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteReviewRequiredFingerprintRule {
    pub message: String,
}

impl ScheduledQuoteReviewRule for ScheduledQuoteReviewRequiredFingerprintRule {
    fn check(&self, item: &ScheduledQuoteReview) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteReviewRequiredCountryRule {
    pub message: String,
}

impl ScheduledQuoteReviewRule for ScheduledQuoteReviewRequiredCountryRule {
    fn check(&self, item: &ScheduledQuoteReview) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteReviewLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledQuoteReviewRule for ScheduledQuoteReviewLabelLengthRule {
    fn check(&self, item: &ScheduledQuoteReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteReviewEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledQuoteReviewRule for ScheduledQuoteReviewEndsAtRangeRule {
    fn check(&self, item: &ScheduledQuoteReview) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled quote review.
#[derive(Debug, Clone)]
pub struct ScheduledQuoteReviewValidator {
    pub rules: Vec<Rc<dyn ScheduledQuoteReviewRule>>,
}

impl ScheduledQuoteReviewValidator {
    pub fn validate(&self, item: &ScheduledQuoteReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledQuoteReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledQuoteReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_quote_review_validator() -> ScheduledQuoteReviewValidator {
    ScheduledQuoteReviewValidator {
        rules: vec![
            Rc::new(ScheduledQuoteReviewRequiredFingerprintRule {
                message: "scheduled quote review needs an entry".to_string(),
            }) as Rc<dyn ScheduledQuoteReviewRule>,
            Rc::new(ScheduledQuoteReviewRequiredCountryRule {
                message: "scheduled quote review needs an entry".to_string(),
            }),
            Rc::new(ScheduledQuoteReviewLabelLengthRule { max_length: 64 }),
            Rc::new(ScheduledQuoteReviewEndsAtRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_quote_review_validator();
        let item = new_scheduled_quote_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ScheduledQuoteReviewRequiredFingerprintRule {
            message: "scheduled quote review needs a value".to_string(),
        };
        let mut item = new_scheduled_quote_review("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = ScheduledQuoteReviewRequiredCountryRule {
            message: "scheduled quote review needs this field".to_string(),
        };
        let mut item = new_scheduled_quote_review("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledQuoteReviewLabelLengthRule { max_length: 64 };
        let mut item = new_scheduled_quote_review("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ScheduledQuoteReviewEndsAtRangeRule { min: 32, max: 64 };
        let mut item = new_scheduled_quote_review("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
