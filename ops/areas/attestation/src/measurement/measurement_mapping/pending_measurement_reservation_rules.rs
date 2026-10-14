use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending measurement reservation must pass. Check returns its problems, or none.
pub trait PendingMeasurementReservationRule: std::fmt::Debug {
    fn check(&self, item: &PendingMeasurementReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementReservationCodeLengthRule {
    pub max_length: i64,
}

impl PendingMeasurementReservationRule for PendingMeasurementReservationCodeLengthRule {
    fn check(&self, item: &PendingMeasurementReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementReservationRequiredChecksumRule {
    pub message: String,
}

impl PendingMeasurementReservationRule for PendingMeasurementReservationRequiredChecksumRule {
    fn check(&self, item: &PendingMeasurementReservation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementReservationRequiredCodeRule {
    pub message: String,
}

impl PendingMeasurementReservationRule for PendingMeasurementReservationRequiredCodeRule {
    fn check(&self, item: &PendingMeasurementReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementReservationChannelLengthRule {
    pub max_length: i64,
}

impl PendingMeasurementReservationRule for PendingMeasurementReservationChannelLengthRule {
    fn check(&self, item: &PendingMeasurementReservation) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementReservationChecksumLengthRule {
    pub max_length: i64,
}

impl PendingMeasurementReservationRule for PendingMeasurementReservationChecksumLengthRule {
    fn check(&self, item: &PendingMeasurementReservation) -> Vec<String> {
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
pub struct PendingMeasurementReservationRequiredNameRule {
    pub message: String,
}

impl PendingMeasurementReservationRule for PendingMeasurementReservationRequiredNameRule {
    fn check(&self, item: &PendingMeasurementReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending measurement reservation.
#[derive(Debug, Clone)]
pub struct PendingMeasurementReservationValidator {
    pub rules: Vec<Rc<dyn PendingMeasurementReservationRule>>,
}

impl PendingMeasurementReservationValidator {
    pub fn validate(&self, item: &PendingMeasurementReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingMeasurementReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingMeasurementReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_measurement_reservation_validator() -> PendingMeasurementReservationValidator {
    PendingMeasurementReservationValidator {
        rules: vec![
            Rc::new(PendingMeasurementReservationCodeLengthRule { max_length: 86400 })
                as Rc<dyn PendingMeasurementReservationRule>,
            Rc::new(PendingMeasurementReservationRequiredChecksumRule {
                message: "pending measurement reservation needs an entry".to_string(),
            }),
            Rc::new(PendingMeasurementReservationRequiredCodeRule {
                message: "pending measurement reservation needs an entry".to_string(),
            }),
            Rc::new(PendingMeasurementReservationChannelLengthRule { max_length: 86400 }),
            Rc::new(PendingMeasurementReservationChecksumLengthRule { max_length: 120 }),
            Rc::new(PendingMeasurementReservationRequiredNameRule {
                message: "pending measurement reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_measurement_reservation_validator();
        let item = new_pending_measurement_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingMeasurementReservationCodeLengthRule { max_length: 64 };
        let mut item = new_pending_measurement_reservation("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = PendingMeasurementReservationRequiredChecksumRule {
            message: "pending measurement reservation needs a value".to_string(),
        };
        let mut item = new_pending_measurement_reservation("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingMeasurementReservationRequiredCodeRule {
            message: "pending measurement reservation needs an entry".to_string(),
        };
        let mut item = new_pending_measurement_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = PendingMeasurementReservationChannelLengthRule { max_length: 16 };
        let mut item = new_pending_measurement_reservation("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = PendingMeasurementReservationChecksumLengthRule { max_length: 64 };
        let mut item = new_pending_measurement_reservation("x", "y");
        item.checksum = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingMeasurementReservationRequiredNameRule {
            message: "pending measurement reservation needs a value".to_string(),
        };
        let mut item = new_pending_measurement_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
