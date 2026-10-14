use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive data export metric must pass. Check returns its problems, or none.
pub trait AdaptiveDataExportMetricRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveDataExportMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDataExportMetricCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveDataExportMetricRule for AdaptiveDataExportMetricCodeLengthRule {
    fn check(&self, item: &AdaptiveDataExportMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDataExportMetricSummaryLengthRule {
    pub max_length: i64,
}

impl AdaptiveDataExportMetricRule for AdaptiveDataExportMetricSummaryLengthRule {
    fn check(&self, item: &AdaptiveDataExportMetric) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDataExportMetricRequiredLabelRule {
    pub message: String,
}

impl AdaptiveDataExportMetricRule for AdaptiveDataExportMetricRequiredLabelRule {
    fn check(&self, item: &AdaptiveDataExportMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDataExportMetricRequiredSummaryRule {
    pub message: String,
}

impl AdaptiveDataExportMetricRule for AdaptiveDataExportMetricRequiredSummaryRule {
    fn check(&self, item: &AdaptiveDataExportMetric) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDataExportMetricPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveDataExportMetricRule for AdaptiveDataExportMetricPriorityRangeRule {
    fn check(&self, item: &AdaptiveDataExportMetric) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive data export metric.
#[derive(Debug, Clone)]
pub struct AdaptiveDataExportMetricValidator {
    pub rules: Vec<Rc<dyn AdaptiveDataExportMetricRule>>,
}

impl AdaptiveDataExportMetricValidator {
    pub fn validate(&self, item: &AdaptiveDataExportMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveDataExportMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveDataExportMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_data_export_metric_validator() -> AdaptiveDataExportMetricValidator {
    AdaptiveDataExportMetricValidator {
        rules: vec![
            Rc::new(AdaptiveDataExportMetricCodeLengthRule { max_length: 64 }) as Rc<dyn AdaptiveDataExportMetricRule>,
            Rc::new(AdaptiveDataExportMetricSummaryLengthRule { max_length: 255 }),
            Rc::new(AdaptiveDataExportMetricRequiredLabelRule {
                message: "adaptive data export metric needs this field".to_string(),
            }),
            Rc::new(AdaptiveDataExportMetricRequiredSummaryRule {
                message: "adaptive data export metric needs this field".to_string(),
            }),
            Rc::new(AdaptiveDataExportMetricPriorityRangeRule { min: 255, max: 86400 }),
        ],
    }
}
