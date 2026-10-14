use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict data export metric must pass. Check returns its problems, or none.
pub trait StrictDataExportMetricRule: std::fmt::Debug {
    fn check(&self, item: &StrictDataExportMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDataExportMetricCodeLengthRule {
    pub max_length: i64,
}

impl StrictDataExportMetricRule for StrictDataExportMetricCodeLengthRule {
    fn check(&self, item: &StrictDataExportMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDataExportMetricRequiredTitleRule {
    pub message: String,
}

impl StrictDataExportMetricRule for StrictDataExportMetricRequiredTitleRule {
    fn check(&self, item: &StrictDataExportMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDataExportMetricCodePrefixRule {
    pub prefix: String,
}

impl StrictDataExportMetricRule for StrictDataExportMetricCodePrefixRule {
    fn check(&self, item: &StrictDataExportMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict data export metric.
#[derive(Debug, Clone)]
pub struct StrictDataExportMetricValidator {
    pub rules: Vec<Rc<dyn StrictDataExportMetricRule>>,
}

impl StrictDataExportMetricValidator {
    pub fn validate(&self, item: &StrictDataExportMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictDataExportMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictDataExportMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_data_export_metric_validator() -> StrictDataExportMetricValidator {
    StrictDataExportMetricValidator {
        rules: vec![
            Rc::new(StrictDataExportMetricCodeLengthRule { max_length: 32 }) as Rc<dyn StrictDataExportMetricRule>,
            Rc::new(StrictDataExportMetricRequiredTitleRule {
                message: "strict data export metric needs a value".to_string(),
            }),
            Rc::new(StrictDataExportMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
