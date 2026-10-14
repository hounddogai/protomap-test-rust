use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant snapshot calculation.
pub trait GrantSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantSnapshotAdjuster {
    pub cap: f64,
}

impl GrantSnapshotAdjuster for CapGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshotRevisionOffsetAdjuster {
    pub weight: f64,
}

impl GrantSnapshotAdjuster for GrantSnapshotRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &GrantSnapshot, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantSnapshotAdjuster {
    pub step: f64,
}

impl GrantSnapshotAdjuster for RoundingGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantSnapshotAdjuster {
    pub minimum: f64,
}

impl GrantSnapshotAdjuster for MinimumGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantSnapshotAdjuster for FlaggedGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantSnapshotAdjuster for TieredGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the grant snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantSnapshotEngine {
    pub steps: Vec<Rc<dyn GrantSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantSnapshotEngine {
    pub fn evaluate(&mut self, item: &GrantSnapshot) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantSnapshotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_grant_snapshot_engine(flags: &flags::Registry) -> GrantSnapshotEngine {
    GrantSnapshotEngine {
        steps: vec![
            Rc::new(CapGrantSnapshotAdjuster { cap: 30884.0 }) as Rc<dyn GrantSnapshotAdjuster>,
            Rc::new(GrantSnapshotRevisionOffsetAdjuster { weight: 0.87 }),
            Rc::new(RoundingGrantSnapshotAdjuster { step: 1.0 }),
            Rc::new(MinimumGrantSnapshotAdjuster { minimum: 7.9 }),
            Rc::new(FlaggedGrantSnapshotAdjuster {
                flags: flags.clone(),
                flag: "grant-snapshot-primary".to_string(),
                percent: 1.6,
            }),
            Rc::new(TieredGrantSnapshotAdjuster {
                bounds: vec![7213.0, 9303.0],
                rates: vec![0.92, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 69423.0,
    }
}
