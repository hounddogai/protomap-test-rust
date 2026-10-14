use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence metric must pass. Check returns its problems, or none.
pub trait SilenceMetricRule: std::fmt::Debug {
    fn check(&self, item: &SilenceMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceMetricRule for SilenceMetricQuotaUnitsRangeRule {
    fn check(&self, item: &SilenceMetric) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricSummaryLengthRule {
    pub max_length: i64,
}

impl SilenceMetricRule for SilenceMetricSummaryLengthRule {
    fn check(&self, item: &SilenceMetric) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricLabelLengthRule {
    pub max_length: i64,
}

impl SilenceMetricRule for SilenceMetricLabelLengthRule {
    fn check(&self, item: &SilenceMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricRequiredIssuerRule {
    pub message: String,
}

impl SilenceMetricRule for SilenceMetricRequiredIssuerRule {
    fn check(&self, item: &SilenceMetric) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SilenceMetricRule for SilenceMetricAllowedLabelRule {
    fn check(&self, item: &SilenceMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricRequiredSummaryRule {
    pub message: String,
}

impl SilenceMetricRule for SilenceMetricRequiredSummaryRule {
    fn check(&self, item: &SilenceMetric) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence metric.
#[derive(Debug, Clone)]
pub struct SilenceMetricValidator {
    pub rules: Vec<Rc<dyn SilenceMetricRule>>,
}

impl SilenceMetricValidator {
    pub fn validate(&self, item: &SilenceMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_metric_validator() -> SilenceMetricValidator {
    SilenceMetricValidator {
        rules: vec![
            Rc::new(SilenceMetricQuotaUnitsRangeRule { min: 255, max: 86400 }) as Rc<dyn SilenceMetricRule>,
            Rc::new(SilenceMetricSummaryLengthRule { max_length: 80 }),
            Rc::new(SilenceMetricLabelLengthRule { max_length: 1000 }),
            Rc::new(SilenceMetricRequiredIssuerRule {
                message: "silence metric needs a value".to_string(),
            }),
            Rc::new(SilenceMetricAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(SilenceMetricRequiredSummaryRule {
                message: "silence metric needs an entry".to_string(),
            }),
        ],
    }
}
