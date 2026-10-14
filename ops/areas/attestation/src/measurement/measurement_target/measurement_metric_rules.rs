use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement metric must pass. Check returns its problems, or none.
pub trait MeasurementMetricRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMetricCodePrefixRule {
    pub prefix: String,
}

impl MeasurementMetricRule for MeasurementMetricCodePrefixRule {
    fn check(&self, item: &MeasurementMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMetricSummaryLengthRule {
    pub max_length: i64,
}

impl MeasurementMetricRule for MeasurementMetricSummaryLengthRule {
    fn check(&self, item: &MeasurementMetric) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMetricRequiredSourceRule {
    pub message: String,
}

impl MeasurementMetricRule for MeasurementMetricRequiredSourceRule {
    fn check(&self, item: &MeasurementMetric) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement metric.
#[derive(Debug, Clone)]
pub struct MeasurementMetricValidator {
    pub rules: Vec<Rc<dyn MeasurementMetricRule>>,
}

impl MeasurementMetricValidator {
    pub fn validate(&self, item: &MeasurementMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_metric_validator() -> MeasurementMetricValidator {
    MeasurementMetricValidator {
        rules: vec![
            Rc::new(MeasurementMetricCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn MeasurementMetricRule>,
            Rc::new(MeasurementMetricSummaryLengthRule { max_length: 120 }),
            Rc::new(MeasurementMetricRequiredSourceRule {
                message: "measurement metric needs a value".to_string(),
            }),
        ],
    }
}
