use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary redaction metric must pass. Check returns its problems, or none.
pub trait PrimaryRedactionMetricRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRedactionMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionMetricLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRedactionMetricRule for PrimaryRedactionMetricLimitRangeRule {
    fn check(&self, item: &PrimaryRedactionMetric) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionMetricCodePrefixRule {
    pub prefix: String,
}

impl PrimaryRedactionMetricRule for PrimaryRedactionMetricCodePrefixRule {
    fn check(&self, item: &PrimaryRedactionMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionMetricTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryRedactionMetricRule for PrimaryRedactionMetricTitleLengthRule {
    fn check(&self, item: &PrimaryRedactionMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary redaction metric.
#[derive(Debug, Clone)]
pub struct PrimaryRedactionMetricValidator {
    pub rules: Vec<Rc<dyn PrimaryRedactionMetricRule>>,
}

impl PrimaryRedactionMetricValidator {
    pub fn validate(&self, item: &PrimaryRedactionMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRedactionMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRedactionMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_redaction_metric_validator() -> PrimaryRedactionMetricValidator {
    PrimaryRedactionMetricValidator {
        rules: vec![
            Rc::new(PrimaryRedactionMetricLimitRangeRule { min: 255, max: 86400 })
                as Rc<dyn PrimaryRedactionMetricRule>,
            Rc::new(PrimaryRedactionMetricCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryRedactionMetricTitleLengthRule { max_length: 80 }),
        ],
    }
}
