use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist snapshot calculation.
pub trait AllowlistSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistSnapshotModifier for TieredAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistSnapshotAdjuster {
    pub step: f64,
}

impl AllowlistSnapshotModifier for RoundingAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistSnapshotAdjuster {
    pub cap: f64,
}

impl AllowlistSnapshotModifier for CapAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSnapshotThresholdOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistSnapshotModifier for AllowlistSnapshotThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistSnapshot, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistSnapshotModifier for FlaggedAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the allowlist snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistSnapshotEngine {
    pub steps: Vec<Rc<dyn AllowlistSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistSnapshotEngine {
    pub fn price(&mut self, item: &AllowlistSnapshot) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistSnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_snapshot_engine(flags: &flags::Registry) -> AllowlistSnapshotEngine {
    AllowlistSnapshotEngine {
        steps: vec![
            Rc::new(TieredAllowlistSnapshotAdjuster {
                bounds: vec![5182.0, 6174.0, 6338.0],
                rates: vec![0.95, 0.86, 0.92],
            }) as Rc<dyn AllowlistSnapshotModifier>,
            Rc::new(RoundingAllowlistSnapshotAdjuster { step: 0.5 }),
            Rc::new(CapAllowlistSnapshotAdjuster { cap: 13163.0 }),
            Rc::new(AllowlistSnapshotThresholdOffsetAdjuster { weight: 0.49 }),
            Rc::new(FlaggedAllowlistSnapshotAdjuster {
                flags: flags.clone(),
                flag: "allowlist-snapshot-adaptive".to_string(),
                percent: 2.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 3984.9,
    }
}
