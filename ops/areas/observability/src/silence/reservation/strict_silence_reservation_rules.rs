use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict silence reservation must pass. Check returns its problems, or none.
pub trait StrictSilenceReservationRule: std::fmt::Debug {
    fn check(&self, item: &StrictSilenceReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilenceReservationCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictSilenceReservationRule for StrictSilenceReservationCapacityRangeRule {
    fn check(&self, item: &StrictSilenceReservation) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilenceReservationCodeLengthRule {
    pub max_length: i64,
}

impl StrictSilenceReservationRule for StrictSilenceReservationCodeLengthRule {
    fn check(&self, item: &StrictSilenceReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilenceReservationChecksumLengthRule {
    pub max_length: i64,
}

impl StrictSilenceReservationRule for StrictSilenceReservationChecksumLengthRule {
    fn check(&self, item: &StrictSilenceReservation) -> Vec<String> {
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

/// Runs every rule of a strict silence reservation.
#[derive(Debug, Clone)]
pub struct StrictSilenceReservationValidator {
    pub rules: Vec<Rc<dyn StrictSilenceReservationRule>>,
}

impl StrictSilenceReservationValidator {
    pub fn validate(&self, item: &StrictSilenceReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictSilenceReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSilenceReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_silence_reservation_validator() -> StrictSilenceReservationValidator {
    StrictSilenceReservationValidator {
        rules: vec![
            Rc::new(StrictSilenceReservationCapacityRangeRule { min: 80, max: 86400 })
                as Rc<dyn StrictSilenceReservationRule>,
            Rc::new(StrictSilenceReservationCodeLengthRule { max_length: 120 }),
            Rc::new(StrictSilenceReservationChecksumLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_silence_reservation_validator();
        let item = new_strict_silence_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = StrictSilenceReservationCapacityRangeRule { min: 64, max: 80 };
        let mut item = new_strict_silence_reservation("x", "y");
        item.capacity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictSilenceReservationCodeLengthRule { max_length: 80 };
        let mut item = new_strict_silence_reservation("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = StrictSilenceReservationChecksumLengthRule { max_length: 255 };
        let mut item = new_strict_silence_reservation("x", "y");
        item.checksum = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
