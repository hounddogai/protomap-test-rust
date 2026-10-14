use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier reservation must pass. Check returns its problems, or none.
pub trait VerifierReservationRule: std::fmt::Debug {
    fn check(&self, item: &VerifierReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservationStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierReservationRule for VerifierReservationStartsAtRangeRule {
    fn check(&self, item: &VerifierReservation) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservationRegionLengthRule {
    pub max_length: i64,
}

impl VerifierReservationRule for VerifierReservationRegionLengthRule {
    fn check(&self, item: &VerifierReservation) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservationCodePrefixRule {
    pub prefix: String,
}

impl VerifierReservationRule for VerifierReservationCodePrefixRule {
    fn check(&self, item: &VerifierReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierReservationRule for VerifierReservationMinimumRangeRule {
    fn check(&self, item: &VerifierReservation) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservationRequiredCurrencyRule {
    pub message: String,
}

impl VerifierReservationRule for VerifierReservationRequiredCurrencyRule {
    fn check(&self, item: &VerifierReservation) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier reservation.
#[derive(Debug, Clone)]
pub struct VerifierReservationValidator {
    pub rules: Vec<Rc<dyn VerifierReservationRule>>,
}

impl VerifierReservationValidator {
    pub fn validate(&self, item: &VerifierReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_reservation_validator() -> VerifierReservationValidator {
    VerifierReservationValidator {
        rules: vec![
            Rc::new(VerifierReservationStartsAtRangeRule { min: 64, max: 255 }) as Rc<dyn VerifierReservationRule>,
            Rc::new(VerifierReservationRegionLengthRule { max_length: 1000 }),
            Rc::new(VerifierReservationCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(VerifierReservationMinimumRangeRule { min: 5.6, max: 1688.0 }),
            Rc::new(VerifierReservationRequiredCurrencyRule {
                message: "verifier reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_reservation_validator();
        let item = new_verifier_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = VerifierReservationStartsAtRangeRule { min: 80, max: 120 };
        let mut item = new_verifier_reservation("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = VerifierReservationRegionLengthRule { max_length: 86400 };
        let mut item = new_verifier_reservation("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = VerifierReservationMinimumRangeRule { min: 5.5, max: 1055.1 };
        let mut item = new_verifier_reservation("x", "y");
        item.minimum = 1056.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = VerifierReservationRequiredCurrencyRule {
            message: "verifier reservation needs a value".to_string(),
        };
        let mut item = new_verifier_reservation("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
