use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement cycle must pass. Check returns its problems, or none.
pub trait MeasurementCycleRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleNameLengthRule {
    pub max_length: i64,
}

impl MeasurementCycleRule for MeasurementCycleNameLengthRule {
    fn check(&self, item: &MeasurementCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementCycleRule for MeasurementCycleCodeLengthRule {
    fn check(&self, item: &MeasurementCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleRequiredNameRule {
    pub message: String,
}

impl MeasurementCycleRule for MeasurementCycleRequiredNameRule {
    fn check(&self, item: &MeasurementCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementCycleRule for MeasurementCycleRatioRangeRule {
    fn check(&self, item: &MeasurementCycle) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl MeasurementCycleRule for MeasurementCycleAllowedLabelRule {
    fn check(&self, item: &MeasurementCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is missing", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleCategoryLengthRule {
    pub max_length: i64,
}

impl MeasurementCycleRule for MeasurementCycleCategoryLengthRule {
    fn check(&self, item: &MeasurementCycle) -> Vec<String> {
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

/// Runs every rule of a measurement cycle.
#[derive(Debug, Clone)]
pub struct MeasurementCycleValidator {
    pub rules: Vec<Rc<dyn MeasurementCycleRule>>,
}

impl MeasurementCycleValidator {
    pub fn validate(&self, item: &MeasurementCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_cycle_validator() -> MeasurementCycleValidator {
    MeasurementCycleValidator {
        rules: vec![
            Rc::new(MeasurementCycleNameLengthRule { max_length: 32 }) as Rc<dyn MeasurementCycleRule>,
            Rc::new(MeasurementCycleCodeLengthRule { max_length: 80 }),
            Rc::new(MeasurementCycleRequiredNameRule {
                message: "measurement cycle needs this field".to_string(),
            }),
            Rc::new(MeasurementCycleRatioRangeRule { min: 2.9, max: 263.2 }),
            Rc::new(MeasurementCycleAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(MeasurementCycleCategoryLengthRule { max_length: 120 }),
        ],
    }
}
