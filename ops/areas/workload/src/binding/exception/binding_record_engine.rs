use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding record calculation.
pub trait BindingRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingRecordStep for TieredBindingRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingRecordAdjuster {
    pub step: f64,
}

impl BindingRecordStep for RoundingBindingRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingRecordAdjuster {
    pub minimum: f64,
}

impl BindingRecordStep for MinimumBindingRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the binding record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingRecordEngine {
    pub steps: Vec<Rc<dyn BindingRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingRecordEngine {
    pub fn compute(&mut self, item: &BindingRecord) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_binding_record_engine() -> BindingRecordEngine {
    BindingRecordEngine {
        steps: vec![
            Rc::new(TieredBindingRecordAdjuster {
                bounds: vec![1577.0, 3492.0],
                rates: vec![0.99, 0.89],
            }) as Rc<dyn BindingRecordStep>,
            Rc::new(RoundingBindingRecordAdjuster { step: 0.25 }),
            Rc::new(MinimumBindingRecordAdjuster { minimum: 14.4 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 50658.5,
    }
}
