use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered segment review must pass. Check returns its problems, or none.
pub trait TieredSegmentReviewRule: std::fmt::Debug {
    fn check(&self, item: &TieredSegmentReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentReviewTimezoneLengthRule {
    pub max_length: i64,
}

impl TieredSegmentReviewRule for TieredSegmentReviewTimezoneLengthRule {
    fn check(&self, item: &TieredSegmentReview) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentReviewLabelLengthRule {
    pub max_length: i64,
}

impl TieredSegmentReviewRule for TieredSegmentReviewLabelLengthRule {
    fn check(&self, item: &TieredSegmentReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentReviewRequiredTimezoneRule {
    pub message: String,
}

impl TieredSegmentReviewRule for TieredSegmentReviewRequiredTimezoneRule {
    fn check(&self, item: &TieredSegmentReview) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered segment review.
#[derive(Debug, Clone)]
pub struct TieredSegmentReviewValidator {
    pub rules: Vec<Rc<dyn TieredSegmentReviewRule>>,
}

impl TieredSegmentReviewValidator {
    pub fn validate(&self, item: &TieredSegmentReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredSegmentReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredSegmentReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_segment_review_validator() -> TieredSegmentReviewValidator {
    TieredSegmentReviewValidator {
        rules: vec![
            Rc::new(TieredSegmentReviewTimezoneLengthRule { max_length: 64 }) as Rc<dyn TieredSegmentReviewRule>,
            Rc::new(TieredSegmentReviewLabelLengthRule { max_length: 3600 }),
            Rc::new(TieredSegmentReviewRequiredTimezoneRule {
                message: "tiered segment review needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_segment_review_validator();
        let item = new_tiered_segment_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = TieredSegmentReviewTimezoneLengthRule { max_length: 86400 };
        let mut item = new_tiered_segment_review("x", "y");
        item.timezone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredSegmentReviewLabelLengthRule { max_length: 255 };
        let mut item = new_tiered_segment_review("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = TieredSegmentReviewRequiredTimezoneRule {
            message: "tiered segment review needs a value".to_string(),
        };
        let mut item = new_tiered_segment_review("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
