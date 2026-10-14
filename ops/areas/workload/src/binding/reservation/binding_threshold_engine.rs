use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the binding threshold calculation.
pub trait BindingThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingThresholdAdjuster {
    pub minimum: f64,
}

impl BindingThresholdFactor for MinimumBindingThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingThresholdAdjuster {
    pub cap: f64,
}

impl BindingThresholdFactor for CapBindingThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingThresholdAdjuster {
    pub percent: f64,
}

impl BindingThresholdFactor for PercentBindingThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the binding threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingThresholdEngine {
    pub steps: Vec<Rc<dyn BindingThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingThresholdEngine {
    pub fn price(&mut self, item: &BindingThreshold) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_binding_threshold_engine() -> BindingThresholdEngine {
    BindingThresholdEngine {
        steps: vec![
            Rc::new(MinimumBindingThresholdAdjuster { minimum: 4.3 }) as Rc<dyn BindingThresholdFactor>,
            Rc::new(CapBindingThresholdAdjuster { cap: 36537.0 }),
            Rc::new(PercentBindingThresholdAdjuster { percent: -19.8 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 31232.5,
    }
}
