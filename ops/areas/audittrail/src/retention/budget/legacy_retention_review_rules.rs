use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy retention review must pass. Check returns its problems, or none.
pub trait LegacyRetentionReviewRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRetentionReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionReviewFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyRetentionReviewRule for LegacyRetentionReviewFloorValueRangeRule {
    fn check(&self, item: &LegacyRetentionReview) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionReviewCodeLengthRule {
    pub max_length: i64,
}

impl LegacyRetentionReviewRule for LegacyRetentionReviewCodeLengthRule {
    fn check(&self, item: &LegacyRetentionReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionReviewRequiredCodeRule {
    pub message: String,
}

impl LegacyRetentionReviewRule for LegacyRetentionReviewRequiredCodeRule {
    fn check(&self, item: &LegacyRetentionReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionReviewRequiredContactEmailRule {
    pub message: String,
}

impl LegacyRetentionReviewRule for LegacyRetentionReviewRequiredContactEmailRule {
    fn check(&self, item: &LegacyRetentionReview) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyRetentionReviewRule for LegacyRetentionReviewAllowedCodeRule {
    fn check(&self, item: &LegacyRetentionReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy retention review.
#[derive(Debug, Clone)]
pub struct LegacyRetentionReviewValidator {
    pub rules: Vec<Rc<dyn LegacyRetentionReviewRule>>,
}

impl LegacyRetentionReviewValidator {
    pub fn validate(&self, item: &LegacyRetentionReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRetentionReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRetentionReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_retention_review_validator() -> LegacyRetentionReviewValidator {
    LegacyRetentionReviewValidator {
        rules: vec![
            Rc::new(LegacyRetentionReviewFloorValueRangeRule { min: 5.9, max: 1562.0 })
                as Rc<dyn LegacyRetentionReviewRule>,
            Rc::new(LegacyRetentionReviewCodeLengthRule { max_length: 3600 }),
            Rc::new(LegacyRetentionReviewRequiredCodeRule {
                message: "legacy retention review needs a value".to_string(),
            }),
            Rc::new(LegacyRetentionReviewRequiredContactEmailRule {
                message: "legacy retention review needs an entry".to_string(),
            }),
            Rc::new(LegacyRetentionReviewAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_retention_review_validator();
        let item = new_legacy_retention_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LegacyRetentionReviewFloorValueRangeRule { min: 6.5, max: 1107.2 };
        let mut item = new_legacy_retention_review("x", "y");
        item.floor_value = 1108.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyRetentionReviewCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_retention_review("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyRetentionReviewRequiredCodeRule {
            message: "legacy retention review needs a value".to_string(),
        };
        let mut item = new_legacy_retention_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = LegacyRetentionReviewRequiredContactEmailRule {
            message: "legacy retention review needs a value".to_string(),
        };
        let mut item = new_legacy_retention_review("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
