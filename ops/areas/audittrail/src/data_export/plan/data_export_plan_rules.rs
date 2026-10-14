use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export plan must pass. Check returns its problems, or none.
pub trait DataExportPlanRule: std::fmt::Debug {
    fn check(&self, item: &DataExportPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPlanRegionLengthRule {
    pub max_length: i64,
}

impl DataExportPlanRule for DataExportPlanRegionLengthRule {
    fn check(&self, item: &DataExportPlan) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPlanCodeLengthRule {
    pub max_length: i64,
}

impl DataExportPlanRule for DataExportPlanCodeLengthRule {
    fn check(&self, item: &DataExportPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportPlanRequiredCountryRule {
    pub message: String,
}

impl DataExportPlanRule for DataExportPlanRequiredCountryRule {
    fn check(&self, item: &DataExportPlan) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export plan.
#[derive(Debug, Clone)]
pub struct DataExportPlanValidator {
    pub rules: Vec<Rc<dyn DataExportPlanRule>>,
}

impl DataExportPlanValidator {
    pub fn validate(&self, item: &DataExportPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_plan_validator() -> DataExportPlanValidator {
    DataExportPlanValidator {
        rules: vec![
            Rc::new(DataExportPlanRegionLengthRule { max_length: 1000 }) as Rc<dyn DataExportPlanRule>,
            Rc::new(DataExportPlanCodeLengthRule { max_length: 80 }),
            Rc::new(DataExportPlanRequiredCountryRule {
                message: "data export plan needs this field".to_string(),
            }),
        ],
    }
}
