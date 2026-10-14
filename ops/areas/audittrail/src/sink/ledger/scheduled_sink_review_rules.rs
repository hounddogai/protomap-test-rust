use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled sink review must pass. Check returns its problems, or none.
pub trait ScheduledSinkReviewRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSinkReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl ScheduledSinkReviewRule for ScheduledSinkReviewAllowedPhoneRule {
    fn check(&self, item: &ScheduledSinkReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is below minimum", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewRequiredCodeRule {
    pub message: String,
}

impl ScheduledSinkReviewRule for ScheduledSinkReviewRequiredCodeRule {
    fn check(&self, item: &ScheduledSinkReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledSinkReviewRule for ScheduledSinkReviewTitleLengthRule {
    fn check(&self, item: &ScheduledSinkReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSinkReviewRule for ScheduledSinkReviewAttemptsRangeRule {
    fn check(&self, item: &ScheduledSinkReview) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewRequiredPhoneRule {
    pub message: String,
}

impl ScheduledSinkReviewRule for ScheduledSinkReviewRequiredPhoneRule {
    fn check(&self, item: &ScheduledSinkReview) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkReviewRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSinkReviewRule for ScheduledSinkReviewRotationDaysRangeRule {
    fn check(&self, item: &ScheduledSinkReview) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled sink review.
#[derive(Debug, Clone)]
pub struct ScheduledSinkReviewValidator {
    pub rules: Vec<Rc<dyn ScheduledSinkReviewRule>>,
}

impl ScheduledSinkReviewValidator {
    pub fn validate(&self, item: &ScheduledSinkReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSinkReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSinkReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_sink_review_validator() -> ScheduledSinkReviewValidator {
    ScheduledSinkReviewValidator {
        rules: vec![
            Rc::new(ScheduledSinkReviewAllowedPhoneRule {
                allowed: vec!["fallback".to_string(), "capped".to_string()],
            }) as Rc<dyn ScheduledSinkReviewRule>,
            Rc::new(ScheduledSinkReviewRequiredCodeRule {
                message: "scheduled sink review needs this field".to_string(),
            }),
            Rc::new(ScheduledSinkReviewTitleLengthRule { max_length: 80 }),
            Rc::new(ScheduledSinkReviewAttemptsRangeRule { min: 64, max: 1000 }),
            Rc::new(ScheduledSinkReviewRequiredPhoneRule {
                message: "scheduled sink review needs a value".to_string(),
            }),
            Rc::new(ScheduledSinkReviewRotationDaysRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_sink_review_validator();
        let item = new_scheduled_sink_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledSinkReviewRequiredCodeRule {
            message: "scheduled sink review needs an entry".to_string(),
        };
        let mut item = new_scheduled_sink_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledSinkReviewTitleLengthRule { max_length: 80 };
        let mut item = new_scheduled_sink_review("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ScheduledSinkReviewAttemptsRangeRule { min: 120, max: 255 };
        let mut item = new_scheduled_sink_review("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ScheduledSinkReviewRequiredPhoneRule {
            message: "scheduled sink review needs this field".to_string(),
        };
        let mut item = new_scheduled_sink_review("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ScheduledSinkReviewRotationDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_scheduled_sink_review("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
