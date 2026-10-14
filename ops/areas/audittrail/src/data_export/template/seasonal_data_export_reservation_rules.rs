use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal data export reservation must pass. Check returns its problems, or none.
pub trait SeasonalDataExportReservationRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDataExportReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservationBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalDataExportReservationRule for SeasonalDataExportReservationBatchSizeRangeRule {
    fn check(&self, item: &SeasonalDataExportReservation) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservationCodePrefixRule {
    pub prefix: String,
}

impl SeasonalDataExportReservationRule for SeasonalDataExportReservationCodePrefixRule {
    fn check(&self, item: &SeasonalDataExportReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservationWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalDataExportReservationRule for SeasonalDataExportReservationWindowDaysRangeRule {
    fn check(&self, item: &SeasonalDataExportReservation) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservationCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalDataExportReservationRule for SeasonalDataExportReservationCodeLengthRule {
    fn check(&self, item: &SeasonalDataExportReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservationRequiredCodeRule {
    pub message: String,
}

impl SeasonalDataExportReservationRule for SeasonalDataExportReservationRequiredCodeRule {
    fn check(&self, item: &SeasonalDataExportReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal data export reservation.
#[derive(Debug, Clone)]
pub struct SeasonalDataExportReservationValidator {
    pub rules: Vec<Rc<dyn SeasonalDataExportReservationRule>>,
}

impl SeasonalDataExportReservationValidator {
    pub fn validate(&self, item: &SeasonalDataExportReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalDataExportReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDataExportReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_data_export_reservation_validator() -> SeasonalDataExportReservationValidator {
    SeasonalDataExportReservationValidator {
        rules: vec![
            Rc::new(SeasonalDataExportReservationBatchSizeRangeRule { min: 1000, max: 3600 })
                as Rc<dyn SeasonalDataExportReservationRule>,
            Rc::new(SeasonalDataExportReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SeasonalDataExportReservationWindowDaysRangeRule { min: 16, max: 64 }),
            Rc::new(SeasonalDataExportReservationCodeLengthRule { max_length: 80 }),
            Rc::new(SeasonalDataExportReservationRequiredCodeRule {
                message: "seasonal data export reservation needs this field".to_string(),
            }),
        ],
    }
}
