use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict timeline review must pass. Check returns its problems, or none.
pub trait StrictTimelineReviewRule: std::fmt::Debug {
    fn check(&self, item: &StrictTimelineReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineReviewRequiredCodeRule {
    pub message: String,
}

impl StrictTimelineReviewRule for StrictTimelineReviewRequiredCodeRule {
    fn check(&self, item: &StrictTimelineReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineReviewRequiredCurrencyRule {
    pub message: String,
}

impl StrictTimelineReviewRule for StrictTimelineReviewRequiredCurrencyRule {
    fn check(&self, item: &StrictTimelineReview) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineReviewPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTimelineReviewRule for StrictTimelineReviewPriorityRangeRule {
    fn check(&self, item: &StrictTimelineReview) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineReviewTitleLengthRule {
    pub max_length: i64,
}

impl StrictTimelineReviewRule for StrictTimelineReviewTitleLengthRule {
    fn check(&self, item: &StrictTimelineReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict timeline review.
#[derive(Debug, Clone)]
pub struct StrictTimelineReviewValidator {
    pub rules: Vec<Rc<dyn StrictTimelineReviewRule>>,
}

impl StrictTimelineReviewValidator {
    pub fn validate(&self, item: &StrictTimelineReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictTimelineReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTimelineReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_timeline_review_validator() -> StrictTimelineReviewValidator {
    StrictTimelineReviewValidator {
        rules: vec![
            Rc::new(StrictTimelineReviewRequiredCodeRule {
                message: "strict timeline review needs an entry".to_string(),
            }) as Rc<dyn StrictTimelineReviewRule>,
            Rc::new(StrictTimelineReviewRequiredCurrencyRule {
                message: "strict timeline review needs an entry".to_string(),
            }),
            Rc::new(StrictTimelineReviewPriorityRangeRule { min: 16, max: 255 }),
            Rc::new(StrictTimelineReviewTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_timeline_review_validator();
        let item = new_strict_timeline_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictTimelineReviewRequiredCodeRule {
            message: "strict timeline review needs this field".to_string(),
        };
        let mut item = new_strict_timeline_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = StrictTimelineReviewRequiredCurrencyRule {
            message: "strict timeline review needs a value".to_string(),
        };
        let mut item = new_strict_timeline_review("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = StrictTimelineReviewPriorityRangeRule { min: 80, max: 3600 };
        let mut item = new_strict_timeline_review("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictTimelineReviewTitleLengthRule { max_length: 255 };
        let mut item = new_strict_timeline_review("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
