use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export allocation must pass. Check returns its problems, or none.
pub trait DataExportAllocationRule: std::fmt::Debug {
    fn check(&self, item: &DataExportAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAllocationRequiredNotesRule {
    pub message: String,
}

impl DataExportAllocationRule for DataExportAllocationRequiredNotesRule {
    fn check(&self, item: &DataExportAllocation) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAllocationRequiredCodeRule {
    pub message: String,
}

impl DataExportAllocationRule for DataExportAllocationRequiredCodeRule {
    fn check(&self, item: &DataExportAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAllocationRequiredChecksumRule {
    pub message: String,
}

impl DataExportAllocationRule for DataExportAllocationRequiredChecksumRule {
    fn check(&self, item: &DataExportAllocation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAllocationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportAllocationRule for DataExportAllocationEndsAtRangeRule {
    fn check(&self, item: &DataExportAllocation) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAllocationDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportAllocationRule for DataExportAllocationDurationMinutesRangeRule {
    fn check(&self, item: &DataExportAllocation) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAllocationWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportAllocationRule for DataExportAllocationWindowDaysRangeRule {
    fn check(&self, item: &DataExportAllocation) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export allocation.
#[derive(Debug, Clone)]
pub struct DataExportAllocationValidator {
    pub rules: Vec<Rc<dyn DataExportAllocationRule>>,
}

impl DataExportAllocationValidator {
    pub fn validate(&self, item: &DataExportAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_allocation_validator() -> DataExportAllocationValidator {
    DataExportAllocationValidator {
        rules: vec![
            Rc::new(DataExportAllocationRequiredNotesRule {
                message: "data export allocation needs an entry".to_string(),
            }) as Rc<dyn DataExportAllocationRule>,
            Rc::new(DataExportAllocationRequiredCodeRule {
                message: "data export allocation needs this field".to_string(),
            }),
            Rc::new(DataExportAllocationRequiredChecksumRule {
                message: "data export allocation needs an entry".to_string(),
            }),
            Rc::new(DataExportAllocationEndsAtRangeRule { min: 120, max: 86400 }),
            Rc::new(DataExportAllocationDurationMinutesRangeRule { min: 64, max: 80 }),
            Rc::new(DataExportAllocationWindowDaysRangeRule { min: 16, max: 120 }),
        ],
    }
}
