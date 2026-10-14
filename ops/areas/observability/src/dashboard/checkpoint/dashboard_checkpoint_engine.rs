use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dashboard checkpoint calculation.
pub trait DashboardCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardCheckpointAdjuster for TieredDashboardCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardCheckpointAdjuster {
    pub minimum: f64,
}

impl DashboardCheckpointAdjuster for MinimumDashboardCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardCheckpointAdjuster {
    pub cap: f64,
}

impl DashboardCheckpointAdjuster for CapDashboardCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the dashboard checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardCheckpointEngine {
    pub steps: Vec<Rc<dyn DashboardCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardCheckpointEngine {
    pub fn evaluate(&mut self, item: &DashboardCheckpoint) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[DashboardCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dashboard_checkpoint_engine() -> DashboardCheckpointEngine {
    DashboardCheckpointEngine {
        steps: vec![
            Rc::new(TieredDashboardCheckpointAdjuster {
                bounds: vec![1553.0, 5427.0, 9362.0],
                rates: vec![1.2, 0.8, 1.0],
            }) as Rc<dyn DashboardCheckpointAdjuster>,
            Rc::new(MinimumDashboardCheckpointAdjuster { minimum: 30.2 }),
            Rc::new(CapDashboardCheckpointAdjuster { cap: 19131.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 79282.1,
    }
}
