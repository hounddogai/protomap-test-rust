use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient ttl policy reservation must pass. Check returns its problems, or none.
pub trait LenientTtlPolicyReservationRule: std::fmt::Debug {
    fn check(&self, item: &LenientTtlPolicyReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyReservationScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientTtlPolicyReservationRule for LenientTtlPolicyReservationScoreRangeRule {
    fn check(&self, item: &LenientTtlPolicyReservation) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyReservationAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl LenientTtlPolicyReservationRule for LenientTtlPolicyReservationAllowedFingerprintRule {
    fn check(&self, item: &LenientTtlPolicyReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is above maximum", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyReservationLabelLengthRule {
    pub max_length: i64,
}

impl LenientTtlPolicyReservationRule for LenientTtlPolicyReservationLabelLengthRule {
    fn check(&self, item: &LenientTtlPolicyReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyReservationCodeLengthRule {
    pub max_length: i64,
}

impl LenientTtlPolicyReservationRule for LenientTtlPolicyReservationCodeLengthRule {
    fn check(&self, item: &LenientTtlPolicyReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyReservationRequiredLabelRule {
    pub message: String,
}

impl LenientTtlPolicyReservationRule for LenientTtlPolicyReservationRequiredLabelRule {
    fn check(&self, item: &LenientTtlPolicyReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyReservationQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTtlPolicyReservationRule for LenientTtlPolicyReservationQuotaUnitsRangeRule {
    fn check(&self, item: &LenientTtlPolicyReservation) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient ttl policy reservation.
#[derive(Debug, Clone)]
pub struct LenientTtlPolicyReservationValidator {
    pub rules: Vec<Rc<dyn LenientTtlPolicyReservationRule>>,
}

impl LenientTtlPolicyReservationValidator {
    pub fn validate(&self, item: &LenientTtlPolicyReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTtlPolicyReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTtlPolicyReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_ttl_policy_reservation_validator() -> LenientTtlPolicyReservationValidator {
    LenientTtlPolicyReservationValidator {
        rules: vec![
            Rc::new(LenientTtlPolicyReservationScoreRangeRule { min: 5.9, max: 141.2 })
                as Rc<dyn LenientTtlPolicyReservationRule>,
            Rc::new(LenientTtlPolicyReservationAllowedFingerprintRule {
                allowed: vec!["pending".to_string(), "primary".to_string(), "capped".to_string()],
            }),
            Rc::new(LenientTtlPolicyReservationLabelLengthRule { max_length: 80 }),
            Rc::new(LenientTtlPolicyReservationCodeLengthRule { max_length: 86400 }),
            Rc::new(LenientTtlPolicyReservationRequiredLabelRule {
                message: "lenient ttl policy reservation needs a value".to_string(),
            }),
            Rc::new(LenientTtlPolicyReservationQuotaUnitsRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_ttl_policy_reservation_validator();
        let item = new_lenient_ttl_policy_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = LenientTtlPolicyReservationScoreRangeRule { min: 6.3, max: 2009.7 };
        let mut item = new_lenient_ttl_policy_reservation("x", "y");
        item.score = 2011.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientTtlPolicyReservationLabelLengthRule { max_length: 86400 };
        let mut item = new_lenient_ttl_policy_reservation("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientTtlPolicyReservationCodeLengthRule { max_length: 16 };
        let mut item = new_lenient_ttl_policy_reservation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientTtlPolicyReservationRequiredLabelRule {
            message: "lenient ttl policy reservation needs an entry".to_string(),
        };
        let mut item = new_lenient_ttl_policy_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LenientTtlPolicyReservationQuotaUnitsRangeRule { min: 32, max: 64 };
        let mut item = new_lenient_ttl_policy_reservation("x", "y");
        item.quota_units = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
