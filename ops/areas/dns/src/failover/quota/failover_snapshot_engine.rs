use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover snapshot calculation.
pub trait FailoverSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSnapshotScoreOffsetAdjuster {
    pub weight: f64,
}

impl FailoverSnapshotFactor for FailoverSnapshotScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverSnapshot, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverSnapshotAdjuster {
    pub cap: f64,
}

impl FailoverSnapshotFactor for CapFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverSnapshotFactor for FlaggedFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFailoverSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverSnapshotFactor for UnitFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &FailoverSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the failover snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverSnapshotEngine {
    pub steps: Vec<Rc<dyn FailoverSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverSnapshotEngine {
    pub fn compute(&mut self, item: &FailoverSnapshot) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FailoverSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_failover_snapshot_engine(flags: &flags::Registry) -> FailoverSnapshotEngine {
    FailoverSnapshotEngine {
        steps: vec![
            Rc::new(FailoverSnapshotScoreOffsetAdjuster { weight: 0.63 }) as Rc<dyn FailoverSnapshotFactor>,
            Rc::new(CapFailoverSnapshotAdjuster { cap: 27233.0 }),
            Rc::new(FlaggedFailoverSnapshotAdjuster {
                flags: flags.clone(),
                flag: "failover-snapshot-strict".to_string(),
                percent: 5.5,
            }),
            Rc::new(UnitFailoverSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.87), ("central".to_string(), 1.2)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 86338.1,
    }
}
