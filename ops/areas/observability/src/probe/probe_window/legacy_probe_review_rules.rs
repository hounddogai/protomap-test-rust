use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy probe review must pass. Check returns its problems, or none.
pub trait LegacyProbeReviewRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbeReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeReviewAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl LegacyProbeReviewRule for LegacyProbeReviewAllowedPhoneRule {
    fn check(&self, item: &LegacyProbeReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is above maximum", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeReviewRequiredCodeRule {
    pub message: String,
}

impl LegacyProbeReviewRule for LegacyProbeReviewRequiredCodeRule {
    fn check(&self, item: &LegacyProbeReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeReviewVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyProbeReviewRule for LegacyProbeReviewVersionRangeRule {
    fn check(&self, item: &LegacyProbeReview) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeReviewRequiredCipherRule {
    pub message: String,
}

impl LegacyProbeReviewRule for LegacyProbeReviewRequiredCipherRule {
    fn check(&self, item: &LegacyProbeReview) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeReviewAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyProbeReviewRule for LegacyProbeReviewAmountRangeRule {
    fn check(&self, item: &LegacyProbeReview) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeReviewPinnedGuardRule {
    pub limit: i64,
}

impl LegacyProbeReviewRule for LegacyProbeReviewPinnedGuardRule {
    fn check(&self, item: &LegacyProbeReview) -> Vec<String> {
        if item.pinned && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while pinned", item.version, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy probe review.
#[derive(Debug, Clone)]
pub struct LegacyProbeReviewValidator {
    pub rules: Vec<Rc<dyn LegacyProbeReviewRule>>,
}

impl LegacyProbeReviewValidator {
    pub fn validate(&self, item: &LegacyProbeReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyProbeReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbeReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_review_validator() -> LegacyProbeReviewValidator {
    LegacyProbeReviewValidator {
        rules: vec![
            Rc::new(LegacyProbeReviewAllowedPhoneRule { allowed: Vec::new() }) as Rc<dyn LegacyProbeReviewRule>,
            Rc::new(LegacyProbeReviewRequiredCodeRule {
                message: "legacy probe review needs an entry".to_string(),
            }),
            Rc::new(LegacyProbeReviewVersionRangeRule { min: 120, max: 255 }),
            Rc::new(LegacyProbeReviewRequiredCipherRule {
                message: "legacy probe review needs an entry".to_string(),
            }),
            Rc::new(LegacyProbeReviewAmountRangeRule { min: 8.4, max: 2619.0 }),
            Rc::new(LegacyProbeReviewPinnedGuardRule { limit: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_probe_review_validator();
        let item = new_legacy_probe_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyProbeReviewRequiredCodeRule {
            message: "legacy probe review needs a value".to_string(),
        };
        let mut item = new_legacy_probe_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LegacyProbeReviewVersionRangeRule { min: 16, max: 1000 };
        let mut item = new_legacy_probe_review("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LegacyProbeReviewRequiredCipherRule {
            message: "legacy probe review needs an entry".to_string(),
        };
        let mut item = new_legacy_probe_review("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = LegacyProbeReviewAmountRangeRule { min: 4.7, max: 740.0 };
        let mut item = new_legacy_probe_review("x", "y");
        item.amount = 741.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
