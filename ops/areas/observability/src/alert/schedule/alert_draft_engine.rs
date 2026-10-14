use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the alert draft calculation.
pub trait AlertDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraftBurstOffsetAdjuster {
    pub weight: f64,
}

impl AlertDraftFactor for AlertDraftBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &AlertDraft, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAlertDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertDraftFactor for UnitAlertDraftAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &AlertDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertDraftAdjuster {
    pub percent: f64,
}

impl AlertDraftFactor for PercentAlertDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertDraftFactor for TieredAlertDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the alert draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertDraftEngine {
    pub steps: Vec<Rc<dyn AlertDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertDraftEngine {
    pub fn evaluate(&mut self, item: &AlertDraft) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertDraftFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[AlertDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_draft_engine() -> AlertDraftEngine {
    AlertDraftEngine {
        steps: vec![
            Rc::new(AlertDraftBurstOffsetAdjuster { weight: 1.62 }) as Rc<dyn AlertDraftFactor>,
            Rc::new(UnitAlertDraftAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.88), ("west".to_string(), 0.99)]),
            }),
            Rc::new(PercentAlertDraftAdjuster { percent: -22.0 }),
            Rc::new(TieredAlertDraftAdjuster {
                bounds: vec![4275.0, 4934.0],
                rates: vec![0.84, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 10865.6,
    }
}
