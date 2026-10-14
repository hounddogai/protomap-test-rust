use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export record must pass. Check returns its problems, or none.
pub trait DataExportRecordRule: std::fmt::Debug {
    fn check(&self, item: &DataExportRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecordGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportRecordRule for DataExportRecordGraceDaysRangeRule {
    fn check(&self, item: &DataExportRecord) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecordCodeLengthRule {
    pub max_length: i64,
}

impl DataExportRecordRule for DataExportRecordCodeLengthRule {
    fn check(&self, item: &DataExportRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecordCategoryLengthRule {
    pub max_length: i64,
}

impl DataExportRecordRule for DataExportRecordCategoryLengthRule {
    fn check(&self, item: &DataExportRecord) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecordAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportRecordRule for DataExportRecordAttemptsRangeRule {
    fn check(&self, item: &DataExportRecord) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecordNameLengthRule {
    pub max_length: i64,
}

impl DataExportRecordRule for DataExportRecordNameLengthRule {
    fn check(&self, item: &DataExportRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export record.
#[derive(Debug, Clone)]
pub struct DataExportRecordValidator {
    pub rules: Vec<Rc<dyn DataExportRecordRule>>,
}

impl DataExportRecordValidator {
    pub fn validate(&self, item: &DataExportRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_record_validator() -> DataExportRecordValidator {
    DataExportRecordValidator {
        rules: vec![
            Rc::new(DataExportRecordGraceDaysRangeRule { min: 3600, max: 86400 }) as Rc<dyn DataExportRecordRule>,
            Rc::new(DataExportRecordCodeLengthRule { max_length: 64 }),
            Rc::new(DataExportRecordCategoryLengthRule { max_length: 32 }),
            Rc::new(DataExportRecordAttemptsRangeRule { min: 80, max: 86400 }),
            Rc::new(DataExportRecordNameLengthRule { max_length: 64 }),
        ],
    }
}
