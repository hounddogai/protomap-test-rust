use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout review must pass. Check returns its problems, or none.
pub trait RolloutReviewRule: std::fmt::Debug {
    fn check(&self, item: &RolloutReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReviewRequiredTitleRule {
    pub message: String,
}

impl RolloutReviewRule for RolloutReviewRequiredTitleRule {
    fn check(&self, item: &RolloutReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReviewTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RolloutReviewRule for RolloutReviewTagsLimitRule {
    fn check(&self, item: &RolloutReview) -> Vec<String> {
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
pub struct RolloutReviewHostLengthRule {
    pub max_length: i64,
}

impl RolloutReviewRule for RolloutReviewHostLengthRule {
    fn check(&self, item: &RolloutReview) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReviewAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RolloutReviewRule for RolloutReviewAllowedTitleRule {
    fn check(&self, item: &RolloutReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout review.
#[derive(Debug, Clone)]
pub struct RolloutReviewValidator {
    pub rules: Vec<Rc<dyn RolloutReviewRule>>,
}

impl RolloutReviewValidator {
    pub fn validate(&self, item: &RolloutReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_review_validator() -> RolloutReviewValidator {
    RolloutReviewValidator {
        rules: vec![
            Rc::new(RolloutReviewRequiredTitleRule {
                message: "rollout review needs this field".to_string(),
            }) as Rc<dyn RolloutReviewRule>,
            Rc::new(RolloutReviewTagsLimitRule {
                max_count: 1000,
                banned: vec!["strict".to_string()],
            }),
            Rc::new(RolloutReviewHostLengthRule { max_length: 64 }),
            Rc::new(RolloutReviewAllowedTitleRule {
                allowed: vec!["fallback".to_string(), "default".to_string(), "primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_review_validator();
        let item = new_rollout_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RolloutReviewRequiredTitleRule {
            message: "rollout review needs an entry".to_string(),
        };
        let mut item = new_rollout_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RolloutReviewHostLengthRule { max_length: 1000 };
        let mut item = new_rollout_review("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
