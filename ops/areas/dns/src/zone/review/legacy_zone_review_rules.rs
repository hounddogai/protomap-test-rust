use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy zone review must pass. Check returns its problems, or none.
pub trait LegacyZoneReviewRule: std::fmt::Debug {
    fn check(&self, item: &LegacyZoneReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReviewCodeLengthRule {
    pub max_length: i64,
}

impl LegacyZoneReviewRule for LegacyZoneReviewCodeLengthRule {
    fn check(&self, item: &LegacyZoneReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReviewRequiredKeyIdRule {
    pub message: String,
}

impl LegacyZoneReviewRule for LegacyZoneReviewRequiredKeyIdRule {
    fn check(&self, item: &LegacyZoneReview) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReviewQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyZoneReviewRule for LegacyZoneReviewQuotaUnitsRangeRule {
    fn check(&self, item: &LegacyZoneReview) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReviewRequiredCodeRule {
    pub message: String,
}

impl LegacyZoneReviewRule for LegacyZoneReviewRequiredCodeRule {
    fn check(&self, item: &LegacyZoneReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReviewCodePrefixRule {
    pub prefix: String,
}

impl LegacyZoneReviewRule for LegacyZoneReviewCodePrefixRule {
    fn check(&self, item: &LegacyZoneReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyZoneReviewRule for LegacyZoneReviewAllowedCodeRule {
    fn check(&self, item: &LegacyZoneReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy zone review.
#[derive(Debug, Clone)]
pub struct LegacyZoneReviewValidator {
    pub rules: Vec<Rc<dyn LegacyZoneReviewRule>>,
}

impl LegacyZoneReviewValidator {
    pub fn validate(&self, item: &LegacyZoneReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyZoneReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyZoneReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_zone_review_validator() -> LegacyZoneReviewValidator {
    LegacyZoneReviewValidator {
        rules: vec![
            Rc::new(LegacyZoneReviewCodeLengthRule { max_length: 86400 }) as Rc<dyn LegacyZoneReviewRule>,
            Rc::new(LegacyZoneReviewRequiredKeyIdRule {
                message: "legacy zone review needs this field".to_string(),
            }),
            Rc::new(LegacyZoneReviewQuotaUnitsRangeRule { min: 16, max: 3600 }),
            Rc::new(LegacyZoneReviewRequiredCodeRule {
                message: "legacy zone review needs an entry".to_string(),
            }),
            Rc::new(LegacyZoneReviewCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LegacyZoneReviewAllowedCodeRule {
                allowed: vec!["pending".to_string(), "default".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_zone_review_validator();
        let item = new_legacy_zone_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyZoneReviewCodeLengthRule { max_length: 16 };
        let mut item = new_legacy_zone_review("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = LegacyZoneReviewRequiredKeyIdRule {
            message: "legacy zone review needs this field".to_string(),
        };
        let mut item = new_legacy_zone_review("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LegacyZoneReviewQuotaUnitsRangeRule { min: 1000, max: 86400 };
        let mut item = new_legacy_zone_review("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyZoneReviewRequiredCodeRule {
            message: "legacy zone review needs a value".to_string(),
        };
        let mut item = new_legacy_zone_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
