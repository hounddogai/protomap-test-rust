use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement baseline must pass. Check returns its problems, or none.
pub trait MeasurementBaselineRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaselineAllowedNameRule {
    pub allowed: Vec<String>,
}

impl MeasurementBaselineRule for MeasurementBaselineAllowedNameRule {
    fn check(&self, item: &MeasurementBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaselineCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementBaselineRule for MeasurementBaselineCodeLengthRule {
    fn check(&self, item: &MeasurementBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaselineRequiredUnitRule {
    pub message: String,
}

impl MeasurementBaselineRule for MeasurementBaselineRequiredUnitRule {
    fn check(&self, item: &MeasurementBaseline) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaselineCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementBaselineRule for MeasurementBaselineCeilingRangeRule {
    fn check(&self, item: &MeasurementBaseline) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement baseline.
#[derive(Debug, Clone)]
pub struct MeasurementBaselineValidator {
    pub rules: Vec<Rc<dyn MeasurementBaselineRule>>,
}

impl MeasurementBaselineValidator {
    pub fn validate(&self, item: &MeasurementBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_baseline_validator() -> MeasurementBaselineValidator {
    MeasurementBaselineValidator {
        rules: vec![
            Rc::new(MeasurementBaselineAllowedNameRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn MeasurementBaselineRule>,
            Rc::new(MeasurementBaselineCodeLengthRule { max_length: 32 }),
            Rc::new(MeasurementBaselineRequiredUnitRule {
                message: "measurement baseline needs a value".to_string(),
            }),
            Rc::new(MeasurementBaselineCeilingRangeRule { min: 4.5, max: 3537.4 }),
        ],
    }
}
