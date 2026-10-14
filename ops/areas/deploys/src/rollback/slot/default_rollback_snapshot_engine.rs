use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default rollback snapshot calculation.
pub trait DefaultRollbackSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRollbackSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultRollbackSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultRollbackSnapshotStep for TieredDefaultRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultRollbackSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRollbackSnapshotAdjuster {
    pub step: f64,
}

impl DefaultRollbackSnapshotStep for RoundingDefaultRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRollbackSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackSnapshotBurstOffsetAdjuster {
    pub weight: f64,
}

impl DefaultRollbackSnapshotStep for DefaultRollbackSnapshotBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultRollbackSnapshot, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDefaultRollbackSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRollbackSnapshotStep for RegionDefaultRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &DefaultRollbackSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default rollback snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRollbackSnapshotEngine {
    pub steps: Vec<Rc<dyn DefaultRollbackSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRollbackSnapshotEngine {
    pub fn evaluate(&mut self, item: &DefaultRollbackSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultRollbackSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_rollback_snapshot_engine() -> DefaultRollbackSnapshotEngine {
    DefaultRollbackSnapshotEngine {
        steps: vec![
            Rc::new(TieredDefaultRollbackSnapshotAdjuster {
                bounds: vec![1396.0, 1417.0, 3131.0, 5086.0],
                rates: vec![0.92, 0.93, 1.03, 1.12],
            }) as Rc<dyn DefaultRollbackSnapshotStep>,
            Rc::new(RoundingDefaultRollbackSnapshotAdjuster { step: 0.5 }),
            Rc::new(DefaultRollbackSnapshotBurstOffsetAdjuster { weight: 0.78 }),
            Rc::new(RegionDefaultRollbackSnapshotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.18), ("west".to_string(), 0.97)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 31096.6,
    }
}
