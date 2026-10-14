use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement group must pass. Check returns its problems, or none.
pub trait MeasurementGroupRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementGroupWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementGroupRule for MeasurementGroupWeightRangeRule {
    fn check(&self, item: &MeasurementGroup) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementGroupLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementGroupRule for MeasurementGroupLimitRangeRule {
    fn check(&self, item: &MeasurementGroup) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementGroupRequiredChecksumRule {
    pub message: String,
}

impl MeasurementGroupRule for MeasurementGroupRequiredChecksumRule {
    fn check(&self, item: &MeasurementGroup) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementGroupScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MeasurementGroupRule for MeasurementGroupScoreRangeRule {
    fn check(&self, item: &MeasurementGroup) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement group.
#[derive(Debug, Clone)]
pub struct MeasurementGroupValidator {
    pub rules: Vec<Rc<dyn MeasurementGroupRule>>,
}

impl MeasurementGroupValidator {
    pub fn validate(&self, item: &MeasurementGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_group_validator() -> MeasurementGroupValidator {
    MeasurementGroupValidator {
        rules: vec![
            Rc::new(MeasurementGroupWeightRangeRule { min: 4.3, max: 86.6 }) as Rc<dyn MeasurementGroupRule>,
            Rc::new(MeasurementGroupLimitRangeRule { min: 32, max: 86400 }),
            Rc::new(MeasurementGroupRequiredChecksumRule {
                message: "measurement group needs an entry".to_string(),
            }),
            Rc::new(MeasurementGroupScoreRangeRule { min: 7.7, max: 2918.0 }),
        ],
    }
}
