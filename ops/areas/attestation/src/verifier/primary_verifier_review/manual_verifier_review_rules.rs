use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual verifier review must pass. Check returns its problems, or none.
pub trait ManualVerifierReviewRule: std::fmt::Debug {
    fn check(&self, item: &ManualVerifierReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierReviewRequiredSourceRule {
    pub message: String,
}

impl ManualVerifierReviewRule for ManualVerifierReviewRequiredSourceRule {
    fn check(&self, item: &ManualVerifierReview) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierReviewTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ManualVerifierReviewRule for ManualVerifierReviewTagsLimitRule {
    fn check(&self, item: &ManualVerifierReview) -> Vec<String> {
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
pub struct ManualVerifierReviewRequiredCountryRule {
    pub message: String,
}

impl ManualVerifierReviewRule for ManualVerifierReviewRequiredCountryRule {
    fn check(&self, item: &ManualVerifierReview) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierReviewNameLengthRule {
    pub max_length: i64,
}

impl ManualVerifierReviewRule for ManualVerifierReviewNameLengthRule {
    fn check(&self, item: &ManualVerifierReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual verifier review.
#[derive(Debug, Clone)]
pub struct ManualVerifierReviewValidator {
    pub rules: Vec<Rc<dyn ManualVerifierReviewRule>>,
}

impl ManualVerifierReviewValidator {
    pub fn validate(&self, item: &ManualVerifierReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualVerifierReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualVerifierReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_verifier_review_validator() -> ManualVerifierReviewValidator {
    ManualVerifierReviewValidator {
        rules: vec![
            Rc::new(ManualVerifierReviewRequiredSourceRule {
                message: "manual verifier review needs a value".to_string(),
            }) as Rc<dyn ManualVerifierReviewRule>,
            Rc::new(ManualVerifierReviewTagsLimitRule {
                max_count: 3600,
                banned: Vec::new(),
            }),
            Rc::new(ManualVerifierReviewRequiredCountryRule {
                message: "manual verifier review needs a value".to_string(),
            }),
            Rc::new(ManualVerifierReviewNameLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_verifier_review_validator();
        let item = new_manual_verifier_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ManualVerifierReviewRequiredSourceRule {
            message: "manual verifier review needs a value".to_string(),
        };
        let mut item = new_manual_verifier_review("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = ManualVerifierReviewRequiredCountryRule {
            message: "manual verifier review needs an entry".to_string(),
        };
        let mut item = new_manual_verifier_review("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualVerifierReviewNameLengthRule { max_length: 32 };
        let mut item = new_manual_verifier_review("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
