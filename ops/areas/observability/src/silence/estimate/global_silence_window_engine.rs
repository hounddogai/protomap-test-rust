use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global silence window calculation.
pub trait GlobalSilenceWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSilenceWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalSilenceWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalSilenceWindowStep for TieredGlobalSilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalSilenceWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalSilenceWindowAdjuster {
    pub minimum: f64,
}

impl GlobalSilenceWindowStep for MinimumGlobalSilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalSilenceWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalSilenceWindowAdjuster {
    pub cap: f64,
}

impl GlobalSilenceWindowStep for CapGlobalSilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalSilenceWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalSilenceWindowAdjuster {
    pub step: f64,
}

impl GlobalSilenceWindowStep for RoundingGlobalSilenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalSilenceWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global silence window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSilenceWindowEngine {
    pub steps: Vec<Rc<dyn GlobalSilenceWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSilenceWindowEngine {
    pub fn run(&mut self, item: &GlobalSilenceWindow) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalSilenceWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_silence_window_engine() -> GlobalSilenceWindowEngine {
    GlobalSilenceWindowEngine {
        steps: vec![
            Rc::new(TieredGlobalSilenceWindowAdjuster {
                bounds: vec![1531.0, 4243.0, 5144.0, 6021.0],
                rates: vec![0.86, 0.88, 1.19, 1.09],
            }) as Rc<dyn GlobalSilenceWindowStep>,
            Rc::new(MinimumGlobalSilenceWindowAdjuster { minimum: 19.0 }),
            Rc::new(CapGlobalSilenceWindowAdjuster { cap: 15250.0 }),
            Rc::new(RoundingGlobalSilenceWindowAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 10581.6,
    }
}
