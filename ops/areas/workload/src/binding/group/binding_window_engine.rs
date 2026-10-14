use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding window calculation.
pub trait BindingWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingWindowAdjuster {
    pub minimum: f64,
}

impl BindingWindowStep for MinimumBindingWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl BindingWindowStep for BindingWindowGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &BindingWindow, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingWindowAdjuster {
    pub cap: f64,
}

impl BindingWindowStep for CapBindingWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingWindowStep for TieredBindingWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the binding window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingWindowEngine {
    pub steps: Vec<Rc<dyn BindingWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingWindowEngine {
    pub fn evaluate(&mut self, item: &BindingWindow) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BindingWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_window_engine() -> BindingWindowEngine {
    BindingWindowEngine {
        steps: vec![
            Rc::new(MinimumBindingWindowAdjuster { minimum: 16.1 }) as Rc<dyn BindingWindowStep>,
            Rc::new(BindingWindowGraceDaysOffsetAdjuster { weight: 1.66 }),
            Rc::new(CapBindingWindowAdjuster { cap: 45185.0 }),
            Rc::new(TieredBindingWindowAdjuster {
                bounds: vec![372.0, 557.0, 7379.0, 9909.0],
                rates: vec![0.88, 0.85, 1.06, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 59252.4,
    }
}
