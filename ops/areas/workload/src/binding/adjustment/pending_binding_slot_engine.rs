use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending binding slot calculation.
pub trait PendingBindingSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingBindingSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingSlotRetriesOffsetAdjuster {
    pub weight: f64,
}

impl PendingBindingSlotStep for PendingBindingSlotRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &PendingBindingSlot, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingBindingSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingBindingSlotStep for TieredPendingBindingSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingBindingSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingBindingSlotAdjuster {
    pub minimum: f64,
}

impl PendingBindingSlotStep for MinimumPendingBindingSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingBindingSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending binding slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingBindingSlotEngine {
    pub steps: Vec<Rc<dyn PendingBindingSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingBindingSlotEngine {
    pub fn compute(&mut self, item: &PendingBindingSlot) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingBindingSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingBindingSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_binding_slot_engine() -> PendingBindingSlotEngine {
    PendingBindingSlotEngine {
        steps: vec![
            Rc::new(PendingBindingSlotRetriesOffsetAdjuster { weight: 1.91 }) as Rc<dyn PendingBindingSlotStep>,
            Rc::new(TieredPendingBindingSlotAdjuster {
                bounds: vec![5308.0, 6202.0, 9457.0],
                rates: vec![1.15, 1.12, 1.16],
            }),
            Rc::new(MinimumPendingBindingSlotAdjuster { minimum: 16.1 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 52956.1,
    }
}
