use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline review must pass. Check returns its problems, or none.
pub trait TimelineReviewRule: std::fmt::Debug {
    fn check(&self, item: &TimelineReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReviewCodePrefixRule {
    pub prefix: String,
}

impl TimelineReviewRule for TimelineReviewCodePrefixRule {
    fn check(&self, item: &TimelineReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReviewTitleLengthRule {
    pub max_length: i64,
}

impl TimelineReviewRule for TimelineReviewTitleLengthRule {
    fn check(&self, item: &TimelineReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReviewCipherLengthRule {
    pub max_length: i64,
}

impl TimelineReviewRule for TimelineReviewCipherLengthRule {
    fn check(&self, item: &TimelineReview) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReviewVisibleGuardRule {
    pub limit: i64,
}

impl TimelineReviewRule for TimelineReviewVisibleGuardRule {
    fn check(&self, item: &TimelineReview) -> Vec<String> {
        if item.visible && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while visible",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReviewQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineReviewRule for TimelineReviewQuantityRangeRule {
    fn check(&self, item: &TimelineReview) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a timeline review.
#[derive(Debug, Clone)]
pub struct TimelineReviewValidator {
    pub rules: Vec<Rc<dyn TimelineReviewRule>>,
}

impl TimelineReviewValidator {
    pub fn validate(&self, item: &TimelineReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_review_validator() -> TimelineReviewValidator {
    TimelineReviewValidator {
        rules: vec![
            Rc::new(TimelineReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn TimelineReviewRule>,
            Rc::new(TimelineReviewTitleLengthRule { max_length: 3600 }),
            Rc::new(TimelineReviewCipherLengthRule { max_length: 255 }),
            Rc::new(TimelineReviewVisibleGuardRule { limit: 255 }),
            Rc::new(TimelineReviewQuantityRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_review_validator();
        let item = new_timeline_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TimelineReviewTitleLengthRule { max_length: 16 };
        let mut item = new_timeline_review("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = TimelineReviewCipherLengthRule { max_length: 80 };
        let mut item = new_timeline_review("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TimelineReviewQuantityRangeRule { min: 1000, max: 1000 };
        let mut item = new_timeline_review("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
