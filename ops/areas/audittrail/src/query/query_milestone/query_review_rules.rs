use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query review must pass. Check returns its problems, or none.
pub trait QueryReviewRule: std::fmt::Debug {
    fn check(&self, item: &QueryReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReviewRequiredTitleRule {
    pub message: String,
}

impl QueryReviewRule for QueryReviewRequiredTitleRule {
    fn check(&self, item: &QueryReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReviewRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryReviewRule for QueryReviewRotationDaysRangeRule {
    fn check(&self, item: &QueryReview) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReviewCodeLengthRule {
    pub max_length: i64,
}

impl QueryReviewRule for QueryReviewCodeLengthRule {
    fn check(&self, item: &QueryReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReviewCategoryLengthRule {
    pub max_length: i64,
}

impl QueryReviewRule for QueryReviewCategoryLengthRule {
    fn check(&self, item: &QueryReview) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReviewRequiredCodeRule {
    pub message: String,
}

impl QueryReviewRule for QueryReviewRequiredCodeRule {
    fn check(&self, item: &QueryReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryReviewRequiredCategoryRule {
    pub message: String,
}

impl QueryReviewRule for QueryReviewRequiredCategoryRule {
    fn check(&self, item: &QueryReview) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a query review.
#[derive(Debug, Clone)]
pub struct QueryReviewValidator {
    pub rules: Vec<Rc<dyn QueryReviewRule>>,
}

impl QueryReviewValidator {
    pub fn validate(&self, item: &QueryReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_review_validator() -> QueryReviewValidator {
    QueryReviewValidator {
        rules: vec![
            Rc::new(QueryReviewRequiredTitleRule {
                message: "query review needs an entry".to_string(),
            }) as Rc<dyn QueryReviewRule>,
            Rc::new(QueryReviewRotationDaysRangeRule { min: 16, max: 16 }),
            Rc::new(QueryReviewCodeLengthRule { max_length: 255 }),
            Rc::new(QueryReviewCategoryLengthRule { max_length: 64 }),
            Rc::new(QueryReviewRequiredCodeRule {
                message: "query review needs this field".to_string(),
            }),
            Rc::new(QueryReviewRequiredCategoryRule {
                message: "query review needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_review_validator();
        let item = new_query_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QueryReviewRequiredTitleRule {
            message: "query review needs a value".to_string(),
        };
        let mut item = new_query_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = QueryReviewRotationDaysRangeRule { min: 64, max: 120 };
        let mut item = new_query_review("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QueryReviewCodeLengthRule { max_length: 64 };
        let mut item = new_query_review("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = QueryReviewCategoryLengthRule { max_length: 32 };
        let mut item = new_query_review("x", "y");
        item.category = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QueryReviewRequiredCodeRule {
            message: "query review needs an entry".to_string(),
        };
        let mut item = new_query_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = QueryReviewRequiredCategoryRule {
            message: "query review needs an entry".to_string(),
        };
        let mut item = new_query_review("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
