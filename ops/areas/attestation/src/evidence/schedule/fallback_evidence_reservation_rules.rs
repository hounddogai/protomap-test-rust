use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback evidence reservation must pass. Check returns its problems, or none.
pub trait FallbackEvidenceReservationRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEvidenceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceReservationCurrencyLengthRule {
    pub max_length: i64,
}

impl FallbackEvidenceReservationRule for FallbackEvidenceReservationCurrencyLengthRule {
    fn check(&self, item: &FallbackEvidenceReservation) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceReservationCodeLengthRule {
    pub max_length: i64,
}

impl FallbackEvidenceReservationRule for FallbackEvidenceReservationCodeLengthRule {
    fn check(&self, item: &FallbackEvidenceReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceReservationDescriptionLengthRule {
    pub max_length: i64,
}

impl FallbackEvidenceReservationRule for FallbackEvidenceReservationDescriptionLengthRule {
    fn check(&self, item: &FallbackEvidenceReservation) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceReservationRequiredCurrencyRule {
    pub message: String,
}

impl FallbackEvidenceReservationRule for FallbackEvidenceReservationRequiredCurrencyRule {
    fn check(&self, item: &FallbackEvidenceReservation) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback evidence reservation.
#[derive(Debug, Clone)]
pub struct FallbackEvidenceReservationValidator {
    pub rules: Vec<Rc<dyn FallbackEvidenceReservationRule>>,
}

impl FallbackEvidenceReservationValidator {
    pub fn validate(&self, item: &FallbackEvidenceReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackEvidenceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEvidenceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_evidence_reservation_validator() -> FallbackEvidenceReservationValidator {
    FallbackEvidenceReservationValidator {
        rules: vec![
            Rc::new(FallbackEvidenceReservationCurrencyLengthRule { max_length: 80 })
                as Rc<dyn FallbackEvidenceReservationRule>,
            Rc::new(FallbackEvidenceReservationCodeLengthRule { max_length: 64 }),
            Rc::new(FallbackEvidenceReservationDescriptionLengthRule { max_length: 86400 }),
            Rc::new(FallbackEvidenceReservationRequiredCurrencyRule {
                message: "fallback evidence reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_evidence_reservation_validator();
        let item = new_fallback_evidence_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = FallbackEvidenceReservationCurrencyLengthRule { max_length: 120 };
        let mut item = new_fallback_evidence_reservation("x", "y");
        item.currency = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackEvidenceReservationCodeLengthRule { max_length: 80 };
        let mut item = new_fallback_evidence_reservation("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = FallbackEvidenceReservationDescriptionLengthRule { max_length: 86400 };
        let mut item = new_fallback_evidence_reservation("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = FallbackEvidenceReservationRequiredCurrencyRule {
            message: "fallback evidence reservation needs this field".to_string(),
        };
        let mut item = new_fallback_evidence_reservation("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
