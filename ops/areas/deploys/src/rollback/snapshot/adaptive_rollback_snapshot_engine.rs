use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive rollback snapshot calculation.
pub trait AdaptiveRollbackSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRollbackSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRollbackSnapshotAdjuster {
    pub percent: f64,
}

impl AdaptiveRollbackSnapshotAdjuster for PercentAdaptiveRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRollbackSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryAdaptiveRollbackSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRollbackSnapshotAdjuster for CategoryAdaptiveRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &AdaptiveRollbackSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRollbackSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRollbackSnapshotAdjuster for TieredAdaptiveRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRollbackSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRollbackSnapshotAdjuster {
    pub cap: f64,
}

impl AdaptiveRollbackSnapshotAdjuster for CapAdaptiveRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRollbackSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackSnapshotMarginOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRollbackSnapshotAdjuster for AdaptiveRollbackSnapshotMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRollbackSnapshot, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRollbackSnapshotAdjuster {
    pub minimum: f64,
}

impl AdaptiveRollbackSnapshotAdjuster for MinimumAdaptiveRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRollbackSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive rollback snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRollbackSnapshotEngine {
    pub steps: Vec<Rc<dyn AdaptiveRollbackSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRollbackSnapshotEngine {
    pub fn evaluate(&mut self, item: &AdaptiveRollbackSnapshot) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveRollbackSnapshotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRollbackSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_rollback_snapshot_engine() -> AdaptiveRollbackSnapshotEngine {
    AdaptiveRollbackSnapshotEngine {
        steps: vec![
            Rc::new(PercentAdaptiveRollbackSnapshotAdjuster { percent: 0.9 })
                as Rc<dyn AdaptiveRollbackSnapshotAdjuster>,
            Rc::new(CategoryAdaptiveRollbackSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.32),
                    ("central".to_string(), 1.26),
                    ("retail".to_string(), 0.86),
                ]),
            }),
            Rc::new(TieredAdaptiveRollbackSnapshotAdjuster {
                bounds: vec![3336.0, 5974.0, 9982.0],
                rates: vec![0.87, 0.81, 1.12],
            }),
            Rc::new(CapAdaptiveRollbackSnapshotAdjuster { cap: 40088.0 }),
            Rc::new(AdaptiveRollbackSnapshotMarginOffsetAdjuster { weight: 1.56 }),
            Rc::new(MinimumAdaptiveRollbackSnapshotAdjuster { minimum: 30.4 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 17254.4,
    }
}
