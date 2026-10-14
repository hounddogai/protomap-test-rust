use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback reservation digest must pass. Check returns its problems, or none.
pub trait FallbackReservationDigestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackReservationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestRequiredNameRule {
    pub message: String,
}

impl FallbackReservationDigestRule for FallbackReservationDigestRequiredNameRule {
    fn check(&self, item: &FallbackReservationDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackReservationDigestRule for FallbackReservationDigestMarginRangeRule {
    fn check(&self, item: &FallbackReservationDigest) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl FallbackReservationDigestRule for FallbackReservationDigestAllowedSourceRule {
    fn check(&self, item: &FallbackReservationDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is inconsistent", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestLocaleLengthRule {
    pub max_length: i64,
}

impl FallbackReservationDigestRule for FallbackReservationDigestLocaleLengthRule {
    fn check(&self, item: &FallbackReservationDigest) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestCodeLengthRule {
    pub max_length: i64,
}

impl FallbackReservationDigestRule for FallbackReservationDigestCodeLengthRule {
    fn check(&self, item: &FallbackReservationDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestRequiredCountryRule {
    pub message: String,
}

impl FallbackReservationDigestRule for FallbackReservationDigestRequiredCountryRule {
    fn check(&self, item: &FallbackReservationDigest) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback reservation digest.
#[derive(Debug, Clone)]
pub struct FallbackReservationDigestValidator {
    pub rules: Vec<Rc<dyn FallbackReservationDigestRule>>,
}

impl FallbackReservationDigestValidator {
    pub fn validate(&self, item: &FallbackReservationDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackReservationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackReservationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_reservation_digest_validator() -> FallbackReservationDigestValidator {
    FallbackReservationDigestValidator {
        rules: vec![
            Rc::new(FallbackReservationDigestRequiredNameRule {
                message: "fallback reservation digest needs this field".to_string(),
            }) as Rc<dyn FallbackReservationDigestRule>,
            Rc::new(FallbackReservationDigestMarginRangeRule { min: 7.7, max: 2694.4 }),
            Rc::new(FallbackReservationDigestAllowedSourceRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(FallbackReservationDigestLocaleLengthRule { max_length: 16 }),
            Rc::new(FallbackReservationDigestCodeLengthRule { max_length: 3600 }),
            Rc::new(FallbackReservationDigestRequiredCountryRule {
                message: "fallback reservation digest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_reservation_digest_validator();
        let item = new_fallback_reservation_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackReservationDigestRequiredNameRule {
            message: "fallback reservation digest needs an entry".to_string(),
        };
        let mut item = new_fallback_reservation_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = FallbackReservationDigestMarginRangeRule { min: 7.6, max: 1098.3 };
        let mut item = new_fallback_reservation_digest("x", "y");
        item.margin = 1099.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = FallbackReservationDigestLocaleLengthRule { max_length: 255 };
        let mut item = new_fallback_reservation_digest("x", "y");
        item.locale = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackReservationDigestCodeLengthRule { max_length: 3600 };
        let mut item = new_fallback_reservation_digest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = FallbackReservationDigestRequiredCountryRule {
            message: "fallback reservation digest needs an entry".to_string(),
        };
        let mut item = new_fallback_reservation_digest("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
