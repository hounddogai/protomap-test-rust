use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient zone metric must pass. Check returns its problems, or none.
pub trait LenientZoneMetricRule: std::fmt::Debug {
    fn check(&self, item: &LenientZoneMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneMetricAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl LenientZoneMetricRule for LenientZoneMetricAllowedLabelRule {
    fn check(&self, item: &LenientZoneMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneMetricRequiredLabelRule {
    pub message: String,
}

impl LenientZoneMetricRule for LenientZoneMetricRequiredLabelRule {
    fn check(&self, item: &LenientZoneMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneMetricSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientZoneMetricRule for LenientZoneMetricSequenceRangeRule {
    fn check(&self, item: &LenientZoneMetric) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient zone metric.
#[derive(Debug, Clone)]
pub struct LenientZoneMetricValidator {
    pub rules: Vec<Rc<dyn LenientZoneMetricRule>>,
}

impl LenientZoneMetricValidator {
    pub fn validate(&self, item: &LenientZoneMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientZoneMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientZoneMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_zone_metric_validator() -> LenientZoneMetricValidator {
    LenientZoneMetricValidator {
        rules: vec![
            Rc::new(LenientZoneMetricAllowedLabelRule {
                allowed: vec!["manual".to_string(), "adaptive".to_string(), "lenient".to_string()],
            }) as Rc<dyn LenientZoneMetricRule>,
            Rc::new(LenientZoneMetricRequiredLabelRule {
                message: "lenient zone metric needs this field".to_string(),
            }),
            Rc::new(LenientZoneMetricSequenceRangeRule { min: 32, max: 86400 }),
        ],
    }
}
