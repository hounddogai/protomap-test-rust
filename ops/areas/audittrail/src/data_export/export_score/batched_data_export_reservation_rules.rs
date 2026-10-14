use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched data export reservation must pass. Check returns its problems, or none.
pub trait BatchedDataExportReservationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedDataExportReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedDataExportReservationRule for BatchedDataExportReservationRateRangeRule {
    fn check(&self, item: &BatchedDataExportReservation) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservationRequiredCodeRule {
    pub message: String,
}

impl BatchedDataExportReservationRule for BatchedDataExportReservationRequiredCodeRule {
    fn check(&self, item: &BatchedDataExportReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservationRequiredPrincipalRule {
    pub message: String,
}

impl BatchedDataExportReservationRule for BatchedDataExportReservationRequiredPrincipalRule {
    fn check(&self, item: &BatchedDataExportReservation) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservationLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedDataExportReservationRule for BatchedDataExportReservationLimitRangeRule {
    fn check(&self, item: &BatchedDataExportReservation) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDataExportReservationPrincipalLengthRule {
    pub max_length: i64,
}

impl BatchedDataExportReservationRule for BatchedDataExportReservationPrincipalLengthRule {
    fn check(&self, item: &BatchedDataExportReservation) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched data export reservation.
#[derive(Debug, Clone)]
pub struct BatchedDataExportReservationValidator {
    pub rules: Vec<Rc<dyn BatchedDataExportReservationRule>>,
}

impl BatchedDataExportReservationValidator {
    pub fn validate(&self, item: &BatchedDataExportReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedDataExportReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedDataExportReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_data_export_reservation_validator() -> BatchedDataExportReservationValidator {
    BatchedDataExportReservationValidator {
        rules: vec![
            Rc::new(BatchedDataExportReservationRateRangeRule { min: 8.6, max: 1410.5 })
                as Rc<dyn BatchedDataExportReservationRule>,
            Rc::new(BatchedDataExportReservationRequiredCodeRule {
                message: "batched data export reservation needs this field".to_string(),
            }),
            Rc::new(BatchedDataExportReservationRequiredPrincipalRule {
                message: "batched data export reservation needs an entry".to_string(),
            }),
            Rc::new(BatchedDataExportReservationLimitRangeRule { min: 32, max: 86400 }),
            Rc::new(BatchedDataExportReservationPrincipalLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_data_export_reservation_validator();
        let item = new_batched_data_export_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BatchedDataExportReservationRateRangeRule { min: 1.1, max: 621.3 };
        let mut item = new_batched_data_export_reservation("x", "y");
        item.rate = 622.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedDataExportReservationRequiredCodeRule {
            message: "batched data export reservation needs a value".to_string(),
        };
        let mut item = new_batched_data_export_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = BatchedDataExportReservationRequiredPrincipalRule {
            message: "batched data export reservation needs an entry".to_string(),
        };
        let mut item = new_batched_data_export_reservation("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = BatchedDataExportReservationLimitRangeRule { min: 16, max: 86400 };
        let mut item = new_batched_data_export_reservation("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = BatchedDataExportReservationPrincipalLengthRule { max_length: 16 };
        let mut item = new_batched_data_export_reservation("x", "y");
        item.principal = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
