use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling credential snapshot calculation.
pub trait RollingCredentialSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingCredentialSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingCredentialSnapshotAdjuster {
    pub step: f64,
}

impl RollingCredentialSnapshotFactor for RoundingRollingCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingCredentialSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSnapshotUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RollingCredentialSnapshotFactor for RollingCredentialSnapshotUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &RollingCredentialSnapshot, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRollingCredentialSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingCredentialSnapshotFactor for SegmentRollingCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RollingCredentialSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingCredentialSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingCredentialSnapshotFactor for TieredRollingCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingCredentialSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingCredentialSnapshotAdjuster {
    pub percent: f64,
}

impl RollingCredentialSnapshotFactor for PercentRollingCredentialSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingCredentialSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling credential snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingCredentialSnapshotEngine {
    pub steps: Vec<Rc<dyn RollingCredentialSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingCredentialSnapshotEngine {
    pub fn compute(&mut self, item: &RollingCredentialSnapshot) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingCredentialSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_credential_snapshot_engine() -> RollingCredentialSnapshotEngine {
    RollingCredentialSnapshotEngine {
        steps: vec![
            Rc::new(RoundingRollingCredentialSnapshotAdjuster { step: 0.1 }) as Rc<dyn RollingCredentialSnapshotFactor>,
            Rc::new(RollingCredentialSnapshotUpdatedAtOffsetAdjuster { weight: 0.12 }),
            Rc::new(SegmentRollingCredentialSnapshotAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.21)]),
            }),
            Rc::new(TieredRollingCredentialSnapshotAdjuster {
                bounds: vec![7467.0, 8333.0, 8963.0, 9648.0],
                rates: vec![1.09, 1.01, 0.98, 0.89],
            }),
            Rc::new(PercentRollingCredentialSnapshotAdjuster { percent: 22.3 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 3476.9,
    }
}
