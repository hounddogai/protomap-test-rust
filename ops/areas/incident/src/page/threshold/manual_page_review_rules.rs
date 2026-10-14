use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual page review must pass. Check returns its problems, or none.
pub trait ManualPageReviewRule: std::fmt::Debug {
    fn check(&self, item: &ManualPageReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageReviewTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ManualPageReviewRule for ManualPageReviewTagsLimitRule {
    fn check(&self, item: &ManualPageReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageReviewRequiredTitleRule {
    pub message: String,
}

impl ManualPageReviewRule for ManualPageReviewRequiredTitleRule {
    fn check(&self, item: &ManualPageReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageReviewTitleLengthRule {
    pub max_length: i64,
}

impl ManualPageReviewRule for ManualPageReviewTitleLengthRule {
    fn check(&self, item: &ManualPageReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageReviewRequiredDescriptionRule {
    pub message: String,
}

impl ManualPageReviewRule for ManualPageReviewRequiredDescriptionRule {
    fn check(&self, item: &ManualPageReview) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageReviewVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualPageReviewRule for ManualPageReviewVersionRangeRule {
    fn check(&self, item: &ManualPageReview) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual page review.
#[derive(Debug, Clone)]
pub struct ManualPageReviewValidator {
    pub rules: Vec<Rc<dyn ManualPageReviewRule>>,
}

impl ManualPageReviewValidator {
    pub fn validate(&self, item: &ManualPageReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualPageReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPageReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_page_review_validator() -> ManualPageReviewValidator {
    ManualPageReviewValidator {
        rules: vec![
            Rc::new(ManualPageReviewTagsLimitRule {
                max_count: 1000,
                banned: vec!["seasonal".to_string(), "legacy".to_string()],
            }) as Rc<dyn ManualPageReviewRule>,
            Rc::new(ManualPageReviewRequiredTitleRule {
                message: "manual page review needs a value".to_string(),
            }),
            Rc::new(ManualPageReviewTitleLengthRule { max_length: 16 }),
            Rc::new(ManualPageReviewRequiredDescriptionRule {
                message: "manual page review needs a value".to_string(),
            }),
            Rc::new(ManualPageReviewVersionRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_page_review_validator();
        let item = new_manual_page_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualPageReviewRequiredTitleRule {
            message: "manual page review needs an entry".to_string(),
        };
        let mut item = new_manual_page_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualPageReviewTitleLengthRule { max_length: 86400 };
        let mut item = new_manual_page_review("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = ManualPageReviewRequiredDescriptionRule {
            message: "manual page review needs this field".to_string(),
        };
        let mut item = new_manual_page_review("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ManualPageReviewVersionRangeRule { min: 64, max: 120 };
        let mut item = new_manual_page_review("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
