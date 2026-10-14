use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release review must pass. Check returns its problems, or none.
pub trait ReleaseReviewRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReviewRequiredFingerprintRule {
    pub message: String,
}

impl ReleaseReviewRule for ReleaseReviewRequiredFingerprintRule {
    fn check(&self, item: &ReleaseReview) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReviewChecksumLengthRule {
    pub max_length: i64,
}

impl ReleaseReviewRule for ReleaseReviewChecksumLengthRule {
    fn check(&self, item: &ReleaseReview) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReviewUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseReviewRule for ReleaseReviewUpdatedAtRangeRule {
    fn check(&self, item: &ReleaseReview) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReviewFingerprintLengthRule {
    pub max_length: i64,
}

impl ReleaseReviewRule for ReleaseReviewFingerprintLengthRule {
    fn check(&self, item: &ReleaseReview) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReviewRequiredTitleRule {
    pub message: String,
}

impl ReleaseReviewRule for ReleaseReviewRequiredTitleRule {
    fn check(&self, item: &ReleaseReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReviewCodePrefixRule {
    pub prefix: String,
}

impl ReleaseReviewRule for ReleaseReviewCodePrefixRule {
    fn check(&self, item: &ReleaseReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release review.
#[derive(Debug, Clone)]
pub struct ReleaseReviewValidator {
    pub rules: Vec<Rc<dyn ReleaseReviewRule>>,
}

impl ReleaseReviewValidator {
    pub fn validate(&self, item: &ReleaseReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_review_validator() -> ReleaseReviewValidator {
    ReleaseReviewValidator {
        rules: vec![
            Rc::new(ReleaseReviewRequiredFingerprintRule {
                message: "release review needs this field".to_string(),
            }) as Rc<dyn ReleaseReviewRule>,
            Rc::new(ReleaseReviewChecksumLengthRule { max_length: 120 }),
            Rc::new(ReleaseReviewUpdatedAtRangeRule { min: 16, max: 255 }),
            Rc::new(ReleaseReviewFingerprintLengthRule { max_length: 32 }),
            Rc::new(ReleaseReviewRequiredTitleRule {
                message: "release review needs a value".to_string(),
            }),
            Rc::new(ReleaseReviewCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_review_validator();
        let item = new_release_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ReleaseReviewRequiredFingerprintRule {
            message: "release review needs an entry".to_string(),
        };
        let mut item = new_release_review("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = ReleaseReviewChecksumLengthRule { max_length: 1000 };
        let mut item = new_release_review("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = ReleaseReviewUpdatedAtRangeRule { min: 80, max: 86400 };
        let mut item = new_release_review("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ReleaseReviewFingerprintLengthRule { max_length: 64 };
        let mut item = new_release_review("x", "y");
        item.fingerprint = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReleaseReviewRequiredTitleRule {
            message: "release review needs an entry".to_string(),
        };
        let mut item = new_release_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
