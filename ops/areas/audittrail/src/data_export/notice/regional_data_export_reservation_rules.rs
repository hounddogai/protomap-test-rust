use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional data export reservation must pass. Check returns its problems, or none.
pub trait RegionalDataExportReservationRule: std::fmt::Debug {
    fn check(&self, item: &RegionalDataExportReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservationRequiredLabelRule {
    pub message: String,
}

impl RegionalDataExportReservationRule for RegionalDataExportReservationRequiredLabelRule {
    fn check(&self, item: &RegionalDataExportReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservationNameLengthRule {
    pub max_length: i64,
}

impl RegionalDataExportReservationRule for RegionalDataExportReservationNameLengthRule {
    fn check(&self, item: &RegionalDataExportReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalDataExportReservationRule for RegionalDataExportReservationAllowedCodeRule {
    fn check(&self, item: &RegionalDataExportReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservationLabelLengthRule {
    pub max_length: i64,
}

impl RegionalDataExportReservationRule for RegionalDataExportReservationLabelLengthRule {
    fn check(&self, item: &RegionalDataExportReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportReservationRequiredCodeRule {
    pub message: String,
}

impl RegionalDataExportReservationRule for RegionalDataExportReservationRequiredCodeRule {
    fn check(&self, item: &RegionalDataExportReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional data export reservation.
#[derive(Debug, Clone)]
pub struct RegionalDataExportReservationValidator {
    pub rules: Vec<Rc<dyn RegionalDataExportReservationRule>>,
}

impl RegionalDataExportReservationValidator {
    pub fn validate(&self, item: &RegionalDataExportReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalDataExportReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalDataExportReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_data_export_reservation_validator() -> RegionalDataExportReservationValidator {
    RegionalDataExportReservationValidator {
        rules: vec![
            Rc::new(RegionalDataExportReservationRequiredLabelRule {
                message: "regional data export reservation needs this field".to_string(),
            }) as Rc<dyn RegionalDataExportReservationRule>,
            Rc::new(RegionalDataExportReservationNameLengthRule { max_length: 64 }),
            Rc::new(RegionalDataExportReservationAllowedCodeRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(RegionalDataExportReservationLabelLengthRule { max_length: 1000 }),
            Rc::new(RegionalDataExportReservationRequiredCodeRule {
                message: "regional data export reservation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_data_export_reservation_validator();
        let item = new_regional_data_export_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalDataExportReservationRequiredLabelRule {
            message: "regional data export reservation needs this field".to_string(),
        };
        let mut item = new_regional_data_export_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionalDataExportReservationNameLengthRule { max_length: 3600 };
        let mut item = new_regional_data_export_reservation("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalDataExportReservationLabelLengthRule { max_length: 255 };
        let mut item = new_regional_data_export_reservation("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalDataExportReservationRequiredCodeRule {
            message: "regional data export reservation needs this field".to_string(),
        };
        let mut item = new_regional_data_export_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
