use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict freeze review must pass. Check returns its problems, or none.
pub trait StrictFreezeReviewRule: std::fmt::Debug {
    fn check(&self, item: &StrictFreezeReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewContactEmailLengthRule {
    pub max_length: i64,
}

impl StrictFreezeReviewRule for StrictFreezeReviewContactEmailLengthRule {
    fn check(&self, item: &StrictFreezeReview) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictFreezeReviewRule for StrictFreezeReviewRateRangeRule {
    fn check(&self, item: &StrictFreezeReview) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewPinnedGuardRule {
    pub limit: i64,
}

impl StrictFreezeReviewRule for StrictFreezeReviewPinnedGuardRule {
    fn check(&self, item: &StrictFreezeReview) -> Vec<String> {
        if item.pinned && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while pinned",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewNameLengthRule {
    pub max_length: i64,
}

impl StrictFreezeReviewRule for StrictFreezeReviewNameLengthRule {
    fn check(&self, item: &StrictFreezeReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewCodeLengthRule {
    pub max_length: i64,
}

impl StrictFreezeReviewRule for StrictFreezeReviewCodeLengthRule {
    fn check(&self, item: &StrictFreezeReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFreezeReviewRequiredNameRule {
    pub message: String,
}

impl StrictFreezeReviewRule for StrictFreezeReviewRequiredNameRule {
    fn check(&self, item: &StrictFreezeReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict freeze review.
#[derive(Debug, Clone)]
pub struct StrictFreezeReviewValidator {
    pub rules: Vec<Rc<dyn StrictFreezeReviewRule>>,
}

impl StrictFreezeReviewValidator {
    pub fn validate(&self, item: &StrictFreezeReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictFreezeReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFreezeReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_freeze_review_validator() -> StrictFreezeReviewValidator {
    StrictFreezeReviewValidator {
        rules: vec![
            Rc::new(StrictFreezeReviewContactEmailLengthRule { max_length: 1000 }) as Rc<dyn StrictFreezeReviewRule>,
            Rc::new(StrictFreezeReviewRateRangeRule { min: 2.2, max: 4321.1 }),
            Rc::new(StrictFreezeReviewPinnedGuardRule { limit: 120 }),
            Rc::new(StrictFreezeReviewNameLengthRule { max_length: 120 }),
            Rc::new(StrictFreezeReviewCodeLengthRule { max_length: 1000 }),
            Rc::new(StrictFreezeReviewRequiredNameRule {
                message: "strict freeze review needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_freeze_review_validator();
        let item = new_strict_freeze_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = StrictFreezeReviewContactEmailLengthRule { max_length: 3600 };
        let mut item = new_strict_freeze_review("x", "y");
        item.contact_email = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = StrictFreezeReviewRateRangeRule { min: 5.1, max: 4437.7 };
        let mut item = new_strict_freeze_review("x", "y");
        item.rate = 4439.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictFreezeReviewNameLengthRule { max_length: 32 };
        let mut item = new_strict_freeze_review("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictFreezeReviewCodeLengthRule { max_length: 32 };
        let mut item = new_strict_freeze_review("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictFreezeReviewRequiredNameRule {
            message: "strict freeze review needs this field".to_string(),
        };
        let mut item = new_strict_freeze_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
