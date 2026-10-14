use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending page metric must pass. Check returns its problems, or none.
pub trait PendingPageMetricRule: std::fmt::Debug {
    fn check(&self, item: &PendingPageMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageMetricQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingPageMetricRule for PendingPageMetricQuantityRangeRule {
    fn check(&self, item: &PendingPageMetric) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageMetricLocaleLengthRule {
    pub max_length: i64,
}

impl PendingPageMetricRule for PendingPageMetricLocaleLengthRule {
    fn check(&self, item: &PendingPageMetric) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageMetricLabelLengthRule {
    pub max_length: i64,
}

impl PendingPageMetricRule for PendingPageMetricLabelLengthRule {
    fn check(&self, item: &PendingPageMetric) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageMetricAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingPageMetricRule for PendingPageMetricAllowedCodeRule {
    fn check(&self, item: &PendingPageMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageMetricRequiredLabelRule {
    pub message: String,
}

impl PendingPageMetricRule for PendingPageMetricRequiredLabelRule {
    fn check(&self, item: &PendingPageMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageMetricRequiredTimezoneRule {
    pub message: String,
}

impl PendingPageMetricRule for PendingPageMetricRequiredTimezoneRule {
    fn check(&self, item: &PendingPageMetric) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending page metric.
#[derive(Debug, Clone)]
pub struct PendingPageMetricValidator {
    pub rules: Vec<Rc<dyn PendingPageMetricRule>>,
}

impl PendingPageMetricValidator {
    pub fn validate(&self, item: &PendingPageMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPageMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPageMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_page_metric_validator() -> PendingPageMetricValidator {
    PendingPageMetricValidator {
        rules: vec![
            Rc::new(PendingPageMetricQuantityRangeRule { min: 16, max: 16 }) as Rc<dyn PendingPageMetricRule>,
            Rc::new(PendingPageMetricLocaleLengthRule { max_length: 16 }),
            Rc::new(PendingPageMetricLabelLengthRule { max_length: 255 }),
            Rc::new(PendingPageMetricAllowedCodeRule {
                allowed: vec!["global".to_string(), "global".to_string()],
            }),
            Rc::new(PendingPageMetricRequiredLabelRule {
                message: "pending page metric needs this field".to_string(),
            }),
            Rc::new(PendingPageMetricRequiredTimezoneRule {
                message: "pending page metric needs a value".to_string(),
            }),
        ],
    }
}
