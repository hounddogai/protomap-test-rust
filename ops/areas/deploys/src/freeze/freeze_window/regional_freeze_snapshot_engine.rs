use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional freeze snapshot calculation.
pub trait RegionalFreezeSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFreezeSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalFreezeSnapshotAdjuster {
    pub percent: f64,
}

impl RegionalFreezeSnapshotStep for PercentRegionalFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalFreezeSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalFreezeSnapshotAdjuster {
    pub step: f64,
}

impl RegionalFreezeSnapshotStep for RoundingRegionalFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalFreezeSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalFreezeSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalFreezeSnapshotStep for TieredRegionalFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalFreezeSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional freeze snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFreezeSnapshotEngine {
    pub steps: Vec<Rc<dyn RegionalFreezeSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFreezeSnapshotEngine {
    pub fn price(&mut self, item: &RegionalFreezeSnapshot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalFreezeSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFreezeSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_freeze_snapshot_engine() -> RegionalFreezeSnapshotEngine {
    RegionalFreezeSnapshotEngine {
        steps: vec![
            Rc::new(PercentRegionalFreezeSnapshotAdjuster { percent: 22.0 }) as Rc<dyn RegionalFreezeSnapshotStep>,
            Rc::new(RoundingRegionalFreezeSnapshotAdjuster { step: 1.0 }),
            Rc::new(TieredRegionalFreezeSnapshotAdjuster {
                bounds: vec![859.0, 1474.0, 8820.0, 9589.0],
                rates: vec![0.97, 1.04, 1.11, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 33658.0,
    }
}
