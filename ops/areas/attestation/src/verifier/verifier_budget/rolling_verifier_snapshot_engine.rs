use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling verifier snapshot calculation.
pub trait RollingVerifierSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingVerifierSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingVerifierSnapshotAdjuster {
    pub minimum: f64,
}

impl RollingVerifierSnapshotFactor for MinimumRollingVerifierSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingVerifierSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingVerifierSnapshotAdjuster {
    pub step: f64,
}

impl RollingVerifierSnapshotFactor for RoundingRollingVerifierSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingVerifierSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierSnapshotRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RollingVerifierSnapshotFactor for RollingVerifierSnapshotRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RollingVerifierSnapshot, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingVerifierSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingVerifierSnapshotFactor for FlaggedRollingVerifierSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingVerifierSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingVerifierSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingVerifierSnapshotFactor for TieredRollingVerifierSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingVerifierSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling verifier snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingVerifierSnapshotEngine {
    pub steps: Vec<Rc<dyn RollingVerifierSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingVerifierSnapshotEngine {
    pub fn compute(&mut self, item: &RollingVerifierSnapshot) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingVerifierSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_verifier_snapshot_engine(flags: &flags::Registry) -> RollingVerifierSnapshotEngine {
    RollingVerifierSnapshotEngine {
        steps: vec![
            Rc::new(MinimumRollingVerifierSnapshotAdjuster { minimum: 48.3 }) as Rc<dyn RollingVerifierSnapshotFactor>,
            Rc::new(RoundingRollingVerifierSnapshotAdjuster { step: 1.0 }),
            Rc::new(RollingVerifierSnapshotRetriesOffsetAdjuster { weight: 0.98 }),
            Rc::new(FlaggedRollingVerifierSnapshotAdjuster {
                flags: flags.clone(),
                flag: "rolling-verifier-snapshot-fallback".to_string(),
                percent: -7.4,
            }),
            Rc::new(TieredRollingVerifierSnapshotAdjuster {
                bounds: vec![2849.0, 3446.0, 8802.0],
                rates: vec![0.88, 1.07, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 4409.1,
    }
}
