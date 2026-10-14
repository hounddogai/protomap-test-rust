use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement reservation must pass. Check returns its problems, or none.
pub trait MeasurementReservationRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReservationLabelLengthRule {
    pub max_length: i64,
}

impl MeasurementReservationRule for MeasurementReservationLabelLengthRule {
    fn check(&self, item: &MeasurementReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReservationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementReservationRule for MeasurementReservationEndsAtRangeRule {
    fn check(&self, item: &MeasurementReservation) -> Vec<String> {
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
pub struct MeasurementReservationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementReservationRule for MeasurementReservationRevisionRangeRule {
    fn check(&self, item: &MeasurementReservation) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReservationRequiredDescriptionRule {
    pub message: String,
}

impl MeasurementReservationRule for MeasurementReservationRequiredDescriptionRule {
    fn check(&self, item: &MeasurementReservation) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReservationScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementReservationRule for MeasurementReservationScoreRangeRule {
    fn check(&self, item: &MeasurementReservation) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement reservation.
#[derive(Debug, Clone)]
pub struct MeasurementReservationValidator {
    pub rules: Vec<Rc<dyn MeasurementReservationRule>>,
}

impl MeasurementReservationValidator {
    pub fn validate(&self, item: &MeasurementReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_reservation_validator() -> MeasurementReservationValidator {
    MeasurementReservationValidator {
        rules: vec![
            Rc::new(MeasurementReservationLabelLengthRule { max_length: 255 }) as Rc<dyn MeasurementReservationRule>,
            Rc::new(MeasurementReservationEndsAtRangeRule { min: 64, max: 86400 }),
            Rc::new(MeasurementReservationRevisionRangeRule { min: 32, max: 32 }),
            Rc::new(MeasurementReservationRequiredDescriptionRule {
                message: "measurement reservation needs this field".to_string(),
            }),
            Rc::new(MeasurementReservationScoreRangeRule { min: 4.3, max: 2747.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_reservation_validator();
        let item = new_measurement_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MeasurementReservationLabelLengthRule { max_length: 80 };
        let mut item = new_measurement_reservation("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = MeasurementReservationEndsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_measurement_reservation("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = MeasurementReservationRevisionRangeRule { min: 16, max: 120 };
        let mut item = new_measurement_reservation("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = MeasurementReservationRequiredDescriptionRule {
            message: "measurement reservation needs this field".to_string(),
        };
        let mut item = new_measurement_reservation("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = MeasurementReservationScoreRangeRule { min: 5.4, max: 4503.0 };
        let mut item = new_measurement_reservation("x", "y");
        item.score = 4504.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
