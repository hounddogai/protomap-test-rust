use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress metric must pass. Check returns its problems, or none.
pub trait EgressMetricRule: std::fmt::Debug {
    fn check(&self, item: &EgressMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMetricPinnedGuardRule {
    pub limit: f64,
}

impl EgressMetricRule for EgressMetricPinnedGuardRule {
    fn check(&self, item: &EgressMetric) -> Vec<String> {
        if item.pinned && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while pinned", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMetricRequiredTitleRule {
    pub message: String,
}

impl EgressMetricRule for EgressMetricRequiredTitleRule {
    fn check(&self, item: &EgressMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMetricAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl EgressMetricRule for EgressMetricAllowedCipherRule {
    fn check(&self, item: &EgressMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is not allowed", item.cipher)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressMetricRequiredLabelRule {
    pub message: String,
}

impl EgressMetricRule for EgressMetricRequiredLabelRule {
    fn check(&self, item: &EgressMetric) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress metric.
#[derive(Debug, Clone)]
pub struct EgressMetricValidator {
    pub rules: Vec<Rc<dyn EgressMetricRule>>,
}

impl EgressMetricValidator {
    pub fn validate(&self, item: &EgressMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_metric_validator() -> EgressMetricValidator {
    EgressMetricValidator {
        rules: vec![
            Rc::new(EgressMetricPinnedGuardRule { limit: 8.3 }) as Rc<dyn EgressMetricRule>,
            Rc::new(EgressMetricRequiredTitleRule {
                message: "egress metric needs a value".to_string(),
            }),
            Rc::new(EgressMetricAllowedCipherRule {
                allowed: vec!["pending".to_string(), "legacy".to_string(), "default".to_string()],
            }),
            Rc::new(EgressMetricRequiredLabelRule {
                message: "egress metric needs this field".to_string(),
            }),
        ],
    }
}
