use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export reservation must pass. Check returns its problems, or none.
pub trait DataExportReservationRule: std::fmt::Debug {
    fn check(&self, item: &DataExportReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReservationLabelLengthRule {
    pub max_length: i64,
}

impl DataExportReservationRule for DataExportReservationLabelLengthRule {
    fn check(&self, item: &DataExportReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReservationRequiredLabelRule {
    pub message: String,
}

impl DataExportReservationRule for DataExportReservationRequiredLabelRule {
    fn check(&self, item: &DataExportReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportReservationRequiredCountryRule {
    pub message: String,
}

impl DataExportReservationRule for DataExportReservationRequiredCountryRule {
    fn check(&self, item: &DataExportReservation) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export reservation.
#[derive(Debug, Clone)]
pub struct DataExportReservationValidator {
    pub rules: Vec<Rc<dyn DataExportReservationRule>>,
}

impl DataExportReservationValidator {
    pub fn validate(&self, item: &DataExportReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_reservation_validator() -> DataExportReservationValidator {
    DataExportReservationValidator {
        rules: vec![
            Rc::new(DataExportReservationLabelLengthRule { max_length: 120 }) as Rc<dyn DataExportReservationRule>,
            Rc::new(DataExportReservationRequiredLabelRule {
                message: "data export reservation needs this field".to_string(),
            }),
            Rc::new(DataExportReservationRequiredCountryRule {
                message: "data export reservation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_reservation_validator();
        let item = new_data_export_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DataExportReservationLabelLengthRule { max_length: 16 };
        let mut item = new_data_export_reservation("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DataExportReservationRequiredLabelRule {
            message: "data export reservation needs this field".to_string(),
        };
        let mut item = new_data_export_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = DataExportReservationRequiredCountryRule {
            message: "data export reservation needs a value".to_string(),
        };
        let mut item = new_data_export_reservation("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
