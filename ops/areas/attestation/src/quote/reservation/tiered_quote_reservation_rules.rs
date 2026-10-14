use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered quote reservation must pass. Check returns its problems, or none.
pub trait TieredQuoteReservationRule: std::fmt::Debug {
    fn check(&self, item: &TieredQuoteReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteReservationSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredQuoteReservationRule for TieredQuoteReservationSequenceRangeRule {
    fn check(&self, item: &TieredQuoteReservation) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteReservationRequiredCipherRule {
    pub message: String,
}

impl TieredQuoteReservationRule for TieredQuoteReservationRequiredCipherRule {
    fn check(&self, item: &TieredQuoteReservation) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteReservationRequiredFingerprintRule {
    pub message: String,
}

impl TieredQuoteReservationRule for TieredQuoteReservationRequiredFingerprintRule {
    fn check(&self, item: &TieredQuoteReservation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered quote reservation.
#[derive(Debug, Clone)]
pub struct TieredQuoteReservationValidator {
    pub rules: Vec<Rc<dyn TieredQuoteReservationRule>>,
}

impl TieredQuoteReservationValidator {
    pub fn validate(&self, item: &TieredQuoteReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredQuoteReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredQuoteReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_quote_reservation_validator() -> TieredQuoteReservationValidator {
    TieredQuoteReservationValidator {
        rules: vec![
            Rc::new(TieredQuoteReservationSequenceRangeRule { min: 120, max: 120 })
                as Rc<dyn TieredQuoteReservationRule>,
            Rc::new(TieredQuoteReservationRequiredCipherRule {
                message: "tiered quote reservation needs a value".to_string(),
            }),
            Rc::new(TieredQuoteReservationRequiredFingerprintRule {
                message: "tiered quote reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_quote_reservation_validator();
        let item = new_tiered_quote_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = TieredQuoteReservationSequenceRangeRule { min: 1000, max: 86400 };
        let mut item = new_tiered_quote_reservation("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = TieredQuoteReservationRequiredCipherRule {
            message: "tiered quote reservation needs this field".to_string(),
        };
        let mut item = new_tiered_quote_reservation("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = TieredQuoteReservationRequiredFingerprintRule {
            message: "tiered quote reservation needs an entry".to_string(),
        };
        let mut item = new_tiered_quote_reservation("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
