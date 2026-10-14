use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched zone snapshot calculation.
pub trait BatchedZoneSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedZoneSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedZoneSnapshotAdjuster {
    pub minimum: f64,
}

impl BatchedZoneSnapshotStep for MinimumBatchedZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedZoneSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedZoneSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedZoneSnapshotStep for LabelBatchedZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedZoneSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedZoneSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedZoneSnapshotStep for TieredBatchedZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedZoneSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedZoneSnapshotAdjuster {
    pub step: f64,
}

impl BatchedZoneSnapshotStep for RoundingBatchedZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedZoneSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedZoneSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedZoneSnapshotStep for FlaggedBatchedZoneSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedZoneSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedZoneSnapshotAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl BatchedZoneSnapshotStep for BatchedZoneSnapshotAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedZoneSnapshot, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the batched zone snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedZoneSnapshotEngine {
    pub steps: Vec<Rc<dyn BatchedZoneSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedZoneSnapshotEngine {
    pub fn compute(&mut self, item: &BatchedZoneSnapshot) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedZoneSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_zone_snapshot_engine(flags: &flags::Registry) -> BatchedZoneSnapshotEngine {
    BatchedZoneSnapshotEngine {
        steps: vec![
            Rc::new(MinimumBatchedZoneSnapshotAdjuster { minimum: 15.9 }) as Rc<dyn BatchedZoneSnapshotStep>,
            Rc::new(LabelBatchedZoneSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.09)]),
            }),
            Rc::new(TieredBatchedZoneSnapshotAdjuster {
                bounds: vec![869.0, 6078.0, 7265.0, 7744.0],
                rates: vec![1.13, 1.08, 1.2, 1.18],
            }),
            Rc::new(RoundingBatchedZoneSnapshotAdjuster { step: 1.0 }),
            Rc::new(FlaggedBatchedZoneSnapshotAdjuster {
                flags: flags.clone(),
                flag: "batched-zone-snapshot-lenient".to_string(),
                percent: -1.5,
            }),
            Rc::new(BatchedZoneSnapshotAttemptsOffsetAdjuster { weight: 1.72 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 34560.7,
    }
}
