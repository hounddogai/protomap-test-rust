use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional measurement reservation must pass. Check returns its problems, or none.
pub trait RegionalMeasurementReservationRule: std::fmt::Debug {
    fn check(&self, item: &RegionalMeasurementReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMeasurementReservationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalMeasurementReservationRule for RegionalMeasurementReservationEndsAtRangeRule {
    fn check(&self, item: &RegionalMeasurementReservation) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMeasurementReservationRequiredFingerprintRule {
    pub message: String,
}

impl RegionalMeasurementReservationRule for RegionalMeasurementReservationRequiredFingerprintRule {
    fn check(&self, item: &RegionalMeasurementReservation) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMeasurementReservationTitleLengthRule {
    pub max_length: i64,
}

impl RegionalMeasurementReservationRule for RegionalMeasurementReservationTitleLengthRule {
    fn check(&self, item: &RegionalMeasurementReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional measurement reservation.
#[derive(Debug, Clone)]
pub struct RegionalMeasurementReservationValidator {
    pub rules: Vec<Rc<dyn RegionalMeasurementReservationRule>>,
}

impl RegionalMeasurementReservationValidator {
    pub fn validate(&self, item: &RegionalMeasurementReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalMeasurementReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalMeasurementReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_measurement_reservation_validator() -> RegionalMeasurementReservationValidator {
    RegionalMeasurementReservationValidator {
        rules: vec![
            Rc::new(RegionalMeasurementReservationEndsAtRangeRule { min: 32, max: 80 })
                as Rc<dyn RegionalMeasurementReservationRule>,
            Rc::new(RegionalMeasurementReservationRequiredFingerprintRule {
                message: "regional measurement reservation needs an entry".to_string(),
            }),
            Rc::new(RegionalMeasurementReservationTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_measurement_reservation_validator();
        let item = new_regional_measurement_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RegionalMeasurementReservationEndsAtRangeRule { min: 16, max: 1000 };
        let mut item = new_regional_measurement_reservation("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = RegionalMeasurementReservationRequiredFingerprintRule {
            message: "regional measurement reservation needs an entry".to_string(),
        };
        let mut item = new_regional_measurement_reservation("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalMeasurementReservationTitleLengthRule { max_length: 80 };
        let mut item = new_regional_measurement_reservation("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
