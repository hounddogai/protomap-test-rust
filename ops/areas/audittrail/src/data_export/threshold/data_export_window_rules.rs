use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export window must pass. Check returns its problems, or none.
pub trait DataExportWindowRule: std::fmt::Debug {
    fn check(&self, item: &DataExportWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportWindowRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportWindowRule for DataExportWindowRevisionRangeRule {
    fn check(&self, item: &DataExportWindow) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportWindowNameLengthRule {
    pub max_length: i64,
}

impl DataExportWindowRule for DataExportWindowNameLengthRule {
    fn check(&self, item: &DataExportWindow) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportWindowRequiredCodeRule {
    pub message: String,
}

impl DataExportWindowRule for DataExportWindowRequiredCodeRule {
    fn check(&self, item: &DataExportWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export window.
#[derive(Debug, Clone)]
pub struct DataExportWindowValidator {
    pub rules: Vec<Rc<dyn DataExportWindowRule>>,
}

impl DataExportWindowValidator {
    pub fn validate(&self, item: &DataExportWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_window_validator() -> DataExportWindowValidator {
    DataExportWindowValidator {
        rules: vec![
            Rc::new(DataExportWindowRevisionRangeRule { min: 120, max: 86400 }) as Rc<dyn DataExportWindowRule>,
            Rc::new(DataExportWindowNameLengthRule { max_length: 3600 }),
            Rc::new(DataExportWindowRequiredCodeRule {
                message: "data export window needs an entry".to_string(),
            }),
        ],
    }
}
