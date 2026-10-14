use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export cycle must pass. Check returns its problems, or none.
pub trait DataExportCycleRule: std::fmt::Debug {
    fn check(&self, item: &DataExportCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycleRequiredNameRule {
    pub message: String,
}

impl DataExportCycleRule for DataExportCycleRequiredNameRule {
    fn check(&self, item: &DataExportCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycleRequiredTenantIdRule {
    pub message: String,
}

impl DataExportCycleRule for DataExportCycleRequiredTenantIdRule {
    fn check(&self, item: &DataExportCycle) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycleRequiredScopeRule {
    pub message: String,
}

impl DataExportCycleRule for DataExportCycleRequiredScopeRule {
    fn check(&self, item: &DataExportCycle) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycleNameLengthRule {
    pub max_length: i64,
}

impl DataExportCycleRule for DataExportCycleNameLengthRule {
    fn check(&self, item: &DataExportCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCycleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportCycleRule for DataExportCycleCeilingRangeRule {
    fn check(&self, item: &DataExportCycle) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a data export cycle.
#[derive(Debug, Clone)]
pub struct DataExportCycleValidator {
    pub rules: Vec<Rc<dyn DataExportCycleRule>>,
}

impl DataExportCycleValidator {
    pub fn validate(&self, item: &DataExportCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_cycle_validator() -> DataExportCycleValidator {
    DataExportCycleValidator {
        rules: vec![
            Rc::new(DataExportCycleRequiredNameRule {
                message: "data export cycle needs a value".to_string(),
            }) as Rc<dyn DataExportCycleRule>,
            Rc::new(DataExportCycleRequiredTenantIdRule {
                message: "data export cycle needs a value".to_string(),
            }),
            Rc::new(DataExportCycleRequiredScopeRule {
                message: "data export cycle needs an entry".to_string(),
            }),
            Rc::new(DataExportCycleNameLengthRule { max_length: 3600 }),
            Rc::new(DataExportCycleCeilingRangeRule { min: 9.7, max: 3721.5 }),
        ],
    }
}
