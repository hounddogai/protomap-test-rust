use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled grant snapshot calculation.
pub trait ScheduledGrantSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledGrantSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledGrantSnapshotAdjuster {
    pub cap: f64,
}

impl ScheduledGrantSnapshotStep for CapScheduledGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledGrantSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshotQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledGrantSnapshotStep for ScheduledGrantSnapshotQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledGrantSnapshot, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledGrantSnapshotAdjuster {
    pub minimum: f64,
}

impl ScheduledGrantSnapshotStep for MinimumScheduledGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledGrantSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledGrantSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledGrantSnapshotStep for TieredScheduledGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledGrantSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledGrantSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledGrantSnapshotStep for FlaggedScheduledGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledGrantSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled grant snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledGrantSnapshotEngine {
    pub steps: Vec<Rc<dyn ScheduledGrantSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledGrantSnapshotEngine {
    pub fn price(&mut self, item: &ScheduledGrantSnapshot) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledGrantSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_grant_snapshot_engine(flags: &flags::Registry) -> ScheduledGrantSnapshotEngine {
    ScheduledGrantSnapshotEngine {
        steps: vec![
            Rc::new(CapScheduledGrantSnapshotAdjuster { cap: 20376.0 }) as Rc<dyn ScheduledGrantSnapshotStep>,
            Rc::new(ScheduledGrantSnapshotQuotaUnitsOffsetAdjuster { weight: 1.03 }),
            Rc::new(MinimumScheduledGrantSnapshotAdjuster { minimum: 38.0 }),
            Rc::new(TieredScheduledGrantSnapshotAdjuster {
                bounds: vec![6139.0, 6159.0, 8672.0],
                rates: vec![1.02, 0.85, 0.92],
            }),
            Rc::new(FlaggedScheduledGrantSnapshotAdjuster {
                flags: flags.clone(),
                flag: "scheduled-grant-snapshot-global".to_string(),
                percent: -4.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 4086.4,
    }
}
