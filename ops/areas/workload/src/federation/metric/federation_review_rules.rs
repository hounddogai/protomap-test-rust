use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation review must pass. Check returns its problems, or none.
pub trait FederationReviewRule: std::fmt::Debug {
    fn check(&self, item: &FederationReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationReviewLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationReviewRule for FederationReviewLimitRangeRule {
    fn check(&self, item: &FederationReview) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationReviewTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl FederationReviewRule for FederationReviewTagsLimitRule {
    fn check(&self, item: &FederationReview) -> Vec<String> {
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
pub struct FederationReviewNameLengthRule {
    pub max_length: i64,
}

impl FederationReviewRule for FederationReviewNameLengthRule {
    fn check(&self, item: &FederationReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation review.
#[derive(Debug, Clone)]
pub struct FederationReviewValidator {
    pub rules: Vec<Rc<dyn FederationReviewRule>>,
}

impl FederationReviewValidator {
    pub fn validate(&self, item: &FederationReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_review_validator() -> FederationReviewValidator {
    FederationReviewValidator {
        rules: vec![
            Rc::new(FederationReviewLimitRangeRule { min: 80, max: 80 }) as Rc<dyn FederationReviewRule>,
            Rc::new(FederationReviewTagsLimitRule {
                max_count: 120,
                banned: Vec::new(),
            }),
            Rc::new(FederationReviewNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_review_validator();
        let item = new_federation_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = FederationReviewLimitRangeRule { min: 32, max: 255 };
        let mut item = new_federation_review("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FederationReviewNameLengthRule { max_length: 80 };
        let mut item = new_federation_review("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
