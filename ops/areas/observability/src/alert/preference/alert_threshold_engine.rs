use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert threshold calculation.
pub trait AlertThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertThresholdAdjuster {
    pub step: f64,
}

impl AlertThresholdStep for RoundingAlertThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertThresholdStep for TieredAlertThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertThresholdAdjuster {
    pub cap: f64,
}

impl AlertThresholdStep for CapAlertThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the alert threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertThresholdEngine {
    pub steps: Vec<Rc<dyn AlertThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertThresholdEngine {
    pub fn evaluate(&mut self, item: &AlertThreshold) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_threshold_engine() -> AlertThresholdEngine {
    AlertThresholdEngine {
        steps: vec![
            Rc::new(RoundingAlertThresholdAdjuster { step: 1.0 }) as Rc<dyn AlertThresholdStep>,
            Rc::new(TieredAlertThresholdAdjuster {
                bounds: vec![5045.0, 5555.0, 8355.0],
                rates: vec![0.93, 0.87, 1.0],
            }),
            Rc::new(CapAlertThresholdAdjuster { cap: 30375.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 27785.0,
    }
}
