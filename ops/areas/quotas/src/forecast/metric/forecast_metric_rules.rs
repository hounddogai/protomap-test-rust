use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast metric must pass. Check returns its problems, or none.
pub trait ForecastMetricRule: std::fmt::Debug {
    fn check(&self, item: &ForecastMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMetricBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastMetricRule for ForecastMetricBatchSizeRangeRule {
    fn check(&self, item: &ForecastMetric) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMetricVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastMetricRule for ForecastMetricVersionRangeRule {
    fn check(&self, item: &ForecastMetric) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMetricCodeLengthRule {
    pub max_length: i64,
}

impl ForecastMetricRule for ForecastMetricCodeLengthRule {
    fn check(&self, item: &ForecastMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast metric.
#[derive(Debug, Clone)]
pub struct ForecastMetricValidator {
    pub rules: Vec<Rc<dyn ForecastMetricRule>>,
}

impl ForecastMetricValidator {
    pub fn validate(&self, item: &ForecastMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_metric_validator() -> ForecastMetricValidator {
    ForecastMetricValidator {
        rules: vec![
            Rc::new(ForecastMetricBatchSizeRangeRule { min: 120, max: 255 }) as Rc<dyn ForecastMetricRule>,
            Rc::new(ForecastMetricVersionRangeRule { min: 16, max: 86400 }),
            Rc::new(ForecastMetricCodeLengthRule { max_length: 120 }),
        ],
    }
}
