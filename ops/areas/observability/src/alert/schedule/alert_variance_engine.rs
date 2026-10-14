use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert variance calculation.
pub trait AlertVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertVarianceAdjuster {
    pub minimum: f64,
}

impl AlertVarianceAdjuster for MinimumAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertVarianceAdjuster {
    pub percent: f64,
}

impl AlertVarianceAdjuster for PercentAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertVarianceAdjuster {
    pub step: f64,
}

impl AlertVarianceAdjuster for RoundingAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the alert variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertVarianceEngine {
    pub steps: Vec<Rc<dyn AlertVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertVarianceEngine {
    pub fn run(&mut self, item: &AlertVariance) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[AlertVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_alert_variance_engine() -> AlertVarianceEngine {
    AlertVarianceEngine {
        steps: vec![
            Rc::new(MinimumAlertVarianceAdjuster { minimum: 18.3 }) as Rc<dyn AlertVarianceAdjuster>,
            Rc::new(PercentAlertVarianceAdjuster { percent: -12.6 }),
            Rc::new(RoundingAlertVarianceAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 11894.1,
    }
}
