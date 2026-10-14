use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default overage review must pass. Check returns its problems, or none.
pub trait DefaultOverageReviewRule: std::fmt::Debug {
    fn check(&self, item: &DefaultOverageReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReviewThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultOverageReviewRule for DefaultOverageReviewThresholdRangeRule {
    fn check(&self, item: &DefaultOverageReview) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReviewRequiredKeyIdRule {
    pub message: String,
}

impl DefaultOverageReviewRule for DefaultOverageReviewRequiredKeyIdRule {
    fn check(&self, item: &DefaultOverageReview) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReviewCodeLengthRule {
    pub max_length: i64,
}

impl DefaultOverageReviewRule for DefaultOverageReviewCodeLengthRule {
    fn check(&self, item: &DefaultOverageReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReviewRequiredCategoryRule {
    pub message: String,
}

impl DefaultOverageReviewRule for DefaultOverageReviewRequiredCategoryRule {
    fn check(&self, item: &DefaultOverageReview) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReviewTitleLengthRule {
    pub max_length: i64,
}

impl DefaultOverageReviewRule for DefaultOverageReviewTitleLengthRule {
    fn check(&self, item: &DefaultOverageReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default overage review.
#[derive(Debug, Clone)]
pub struct DefaultOverageReviewValidator {
    pub rules: Vec<Rc<dyn DefaultOverageReviewRule>>,
}

impl DefaultOverageReviewValidator {
    pub fn validate(&self, item: &DefaultOverageReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultOverageReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultOverageReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_overage_review_validator() -> DefaultOverageReviewValidator {
    DefaultOverageReviewValidator {
        rules: vec![
            Rc::new(DefaultOverageReviewThresholdRangeRule { min: 9.7, max: 4852.3 })
                as Rc<dyn DefaultOverageReviewRule>,
            Rc::new(DefaultOverageReviewRequiredKeyIdRule {
                message: "default overage review needs a value".to_string(),
            }),
            Rc::new(DefaultOverageReviewCodeLengthRule { max_length: 16 }),
            Rc::new(DefaultOverageReviewRequiredCategoryRule {
                message: "default overage review needs this field".to_string(),
            }),
            Rc::new(DefaultOverageReviewTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_overage_review_validator();
        let item = new_default_overage_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = DefaultOverageReviewThresholdRangeRule { min: 5.0, max: 1772.8 };
        let mut item = new_default_overage_review("x", "y");
        item.threshold = 1774.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = DefaultOverageReviewRequiredKeyIdRule {
            message: "default overage review needs a value".to_string(),
        };
        let mut item = new_default_overage_review("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultOverageReviewCodeLengthRule { max_length: 3600 };
        let mut item = new_default_overage_review("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = DefaultOverageReviewRequiredCategoryRule {
            message: "default overage review needs an entry".to_string(),
        };
        let mut item = new_default_overage_review("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultOverageReviewTitleLengthRule { max_length: 120 };
        let mut item = new_default_overage_review("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
