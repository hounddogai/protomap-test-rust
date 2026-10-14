use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default plan snapshot calculation.
pub trait DefaultPlanSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPlanSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultPlanSnapshotAdjuster {
    pub cap: f64,
}

impl DefaultPlanSnapshotAdjuster for CapDefaultPlanSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultPlanSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultPlanSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultPlanSnapshotAdjuster for TieredDefaultPlanSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultPlanSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultPlanSnapshotAdjuster {
    pub step: f64,
}

impl DefaultPlanSnapshotAdjuster for RoundingDefaultPlanSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultPlanSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultPlanSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultPlanSnapshotAdjuster for FlaggedDefaultPlanSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultPlanSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default plan snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPlanSnapshotEngine {
    pub steps: Vec<Rc<dyn DefaultPlanSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPlanSnapshotEngine {
    pub fn compute(&mut self, item: &DefaultPlanSnapshot) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultPlanSnapshotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPlanSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_plan_snapshot_engine(flags: &flags::Registry) -> DefaultPlanSnapshotEngine {
    DefaultPlanSnapshotEngine {
        steps: vec![
            Rc::new(CapDefaultPlanSnapshotAdjuster { cap: 10209.0 }) as Rc<dyn DefaultPlanSnapshotAdjuster>,
            Rc::new(TieredDefaultPlanSnapshotAdjuster {
                bounds: vec![394.0, 927.0, 5071.0],
                rates: vec![0.82, 0.87, 1.07],
            }),
            Rc::new(RoundingDefaultPlanSnapshotAdjuster { step: 0.05 }),
            Rc::new(FlaggedDefaultPlanSnapshotAdjuster {
                flags: flags.clone(),
                flag: "default-plan-snapshot-capped".to_string(),
                percent: -9.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 30281.3,
    }
}
