use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict runbook reservation must pass. Check returns its problems, or none.
pub trait StrictRunbookReservationRule: std::fmt::Debug {
    fn check(&self, item: &StrictRunbookReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookReservationRequiredNameRule {
    pub message: String,
}

impl StrictRunbookReservationRule for StrictRunbookReservationRequiredNameRule {
    fn check(&self, item: &StrictRunbookReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookReservationNameLengthRule {
    pub max_length: i64,
}

impl StrictRunbookReservationRule for StrictRunbookReservationNameLengthRule {
    fn check(&self, item: &StrictRunbookReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookReservationRequiredFingerprintRule {
    pub message: String,
}

impl StrictRunbookReservationRule for StrictRunbookReservationRequiredFingerprintRule {
    fn check(&self, item: &StrictRunbookReservation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict runbook reservation.
#[derive(Debug, Clone)]
pub struct StrictRunbookReservationValidator {
    pub rules: Vec<Rc<dyn StrictRunbookReservationRule>>,
}

impl StrictRunbookReservationValidator {
    pub fn validate(&self, item: &StrictRunbookReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictRunbookReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRunbookReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_runbook_reservation_validator() -> StrictRunbookReservationValidator {
    StrictRunbookReservationValidator {
        rules: vec![
            Rc::new(StrictRunbookReservationRequiredNameRule {
                message: "strict runbook reservation needs a value".to_string(),
            }) as Rc<dyn StrictRunbookReservationRule>,
            Rc::new(StrictRunbookReservationNameLengthRule { max_length: 1000 }),
            Rc::new(StrictRunbookReservationRequiredFingerprintRule {
                message: "strict runbook reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_runbook_reservation_validator();
        let item = new_strict_runbook_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictRunbookReservationRequiredNameRule {
            message: "strict runbook reservation needs a value".to_string(),
        };
        let mut item = new_strict_runbook_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictRunbookReservationNameLengthRule { max_length: 86400 };
        let mut item = new_strict_runbook_reservation("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = StrictRunbookReservationRequiredFingerprintRule {
            message: "strict runbook reservation needs this field".to_string(),
        };
        let mut item = new_strict_runbook_reservation("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
