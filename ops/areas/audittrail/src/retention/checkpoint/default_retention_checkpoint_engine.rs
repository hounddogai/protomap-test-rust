use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default retention checkpoint calculation.
pub trait DefaultRetentionCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRetentionCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRetentionCheckpointBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DefaultRetentionCheckpointModifier for DefaultRetentionCheckpointBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultRetentionCheckpoint, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultRetentionCheckpointAdjuster {
    pub cap: f64,
}

impl DefaultRetentionCheckpointModifier for CapDefaultRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultRetentionCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRetentionCheckpointAdjuster {
    pub step: f64,
}

impl DefaultRetentionCheckpointModifier for RoundingDefaultRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRetentionCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultRetentionCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultRetentionCheckpointModifier for TieredDefaultRetentionCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultRetentionCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default retention checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRetentionCheckpointEngine {
    pub steps: Vec<Rc<dyn DefaultRetentionCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRetentionCheckpointEngine {
    pub fn compute(&mut self, item: &DefaultRetentionCheckpoint) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRetentionCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_retention_checkpoint_engine() -> DefaultRetentionCheckpointEngine {
    DefaultRetentionCheckpointEngine {
        steps: vec![
            Rc::new(DefaultRetentionCheckpointBudgetOffsetAdjuster { weight: 0.86 })
                as Rc<dyn DefaultRetentionCheckpointModifier>,
            Rc::new(CapDefaultRetentionCheckpointAdjuster { cap: 30500.0 }),
            Rc::new(RoundingDefaultRetentionCheckpointAdjuster { step: 0.25 }),
            Rc::new(TieredDefaultRetentionCheckpointAdjuster {
                bounds: vec![653.0, 8778.0],
                rates: vec![0.92, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 4305.5,
    }
}
