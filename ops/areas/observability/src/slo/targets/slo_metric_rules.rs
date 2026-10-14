use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo metric must pass. Check returns its problems, or none.
pub trait SloMetricRule: std::fmt::Debug {
    fn check(&self, item: &SloMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMetricCodeLengthRule {
    pub max_length: i64,
}

impl SloMetricRule for SloMetricCodeLengthRule {
    fn check(&self, item: &SloMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMetricRequiredPhoneRule {
    pub message: String,
}

impl SloMetricRule for SloMetricRequiredPhoneRule {
    fn check(&self, item: &SloMetric) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMetricRequiredCodeRule {
    pub message: String,
}

impl SloMetricRule for SloMetricRequiredCodeRule {
    fn check(&self, item: &SloMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMetricPhoneLengthRule {
    pub max_length: i64,
}

impl SloMetricRule for SloMetricPhoneLengthRule {
    fn check(&self, item: &SloMetric) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMetricCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloMetricRule for SloMetricCreatedAtRangeRule {
    fn check(&self, item: &SloMetric) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a slo metric.
#[derive(Debug, Clone)]
pub struct SloMetricValidator {
    pub rules: Vec<Rc<dyn SloMetricRule>>,
}

impl SloMetricValidator {
    pub fn validate(&self, item: &SloMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_metric_validator() -> SloMetricValidator {
    SloMetricValidator {
        rules: vec![
            Rc::new(SloMetricCodeLengthRule { max_length: 86400 }) as Rc<dyn SloMetricRule>,
            Rc::new(SloMetricRequiredPhoneRule {
                message: "slo metric needs this field".to_string(),
            }),
            Rc::new(SloMetricRequiredCodeRule {
                message: "slo metric needs this field".to_string(),
            }),
            Rc::new(SloMetricPhoneLengthRule { max_length: 1000 }),
            Rc::new(SloMetricCreatedAtRangeRule { min: 32, max: 64 }),
        ],
    }
}
