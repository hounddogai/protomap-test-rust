use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the nonce snapshot calculation.
pub trait NonceSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitNonceSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NonceSnapshotModifier for UnitNonceSnapshotAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &NonceSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceSnapshotModifier for FlaggedNonceSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshotCapacityOffsetAdjuster {
    pub weight: f64,
}

impl NonceSnapshotModifier for NonceSnapshotCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &NonceSnapshot, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceSnapshotModifier for TieredNonceSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the nonce snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceSnapshotEngine {
    pub steps: Vec<Rc<dyn NonceSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceSnapshotEngine {
    pub fn evaluate(&mut self, item: &NonceSnapshot) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceSnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_snapshot_engine(flags: &flags::Registry) -> NonceSnapshotEngine {
    NonceSnapshotEngine {
        steps: vec![
            Rc::new(UnitNonceSnapshotAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.05)]),
            }) as Rc<dyn NonceSnapshotModifier>,
            Rc::new(FlaggedNonceSnapshotAdjuster {
                flags: flags.clone(),
                flag: "nonce-snapshot-primary".to_string(),
                percent: -8.1,
            }),
            Rc::new(NonceSnapshotCapacityOffsetAdjuster { weight: 0.75 }),
            Rc::new(TieredNonceSnapshotAdjuster {
                bounds: vec![1105.0, 2531.0, 3906.0],
                rates: vec![0.97, 0.87, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 75931.8,
    }
}
