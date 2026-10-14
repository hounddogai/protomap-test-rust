use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rollback snapshot calculation.
pub trait RollbackSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRollbackSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackSnapshotModifier for SourceRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RollbackSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackSnapshotAdjuster {
    pub cap: f64,
}

impl RollbackSnapshotModifier for CapRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackSnapshotModifier for FlaggedRollbackSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackSnapshotEngine {
    pub steps: Vec<Rc<dyn RollbackSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackSnapshotEngine {
    pub fn compute(&mut self, item: &RollbackSnapshot) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackSnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_snapshot_engine(flags: &flags::Registry) -> RollbackSnapshotEngine {
    RollbackSnapshotEngine {
        steps: vec![
            Rc::new(SourceRollbackSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.15), ("east".to_string(), 0.81)]),
            }) as Rc<dyn RollbackSnapshotModifier>,
            Rc::new(CapRollbackSnapshotAdjuster { cap: 9649.0 }),
            Rc::new(FlaggedRollbackSnapshotAdjuster {
                flags: flags.clone(),
                flag: "rollback-snapshot-rolling".to_string(),
                percent: -3.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 89614.5,
    }
}
