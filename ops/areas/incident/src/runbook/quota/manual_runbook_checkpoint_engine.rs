use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual runbook checkpoint calculation.
pub trait ManualRunbookCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRunbookCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualRunbookCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRunbookCheckpointStep for NameManualRunbookCheckpointAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualRunbookCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookCheckpointScoreOffsetAdjuster {
    pub weight: f64,
}

impl ManualRunbookCheckpointStep for ManualRunbookCheckpointScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRunbookCheckpoint, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRunbookCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRunbookCheckpointStep for FlaggedManualRunbookCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRunbookCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualRunbookCheckpointAdjuster {
    pub percent: f64,
}

impl ManualRunbookCheckpointStep for PercentManualRunbookCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualRunbookCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualRunbookCheckpointAdjuster {
    pub cap: f64,
}

impl ManualRunbookCheckpointStep for CapManualRunbookCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualRunbookCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual runbook checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRunbookCheckpointEngine {
    pub steps: Vec<Rc<dyn ManualRunbookCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRunbookCheckpointEngine {
    pub fn compute(&mut self, item: &ManualRunbookCheckpoint) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualRunbookCheckpointStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualRunbookCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_runbook_checkpoint_engine(flags: &flags::Registry) -> ManualRunbookCheckpointEngine {
    ManualRunbookCheckpointEngine {
        steps: vec![
            Rc::new(NameManualRunbookCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.01),
                    ("north".to_string(), 0.98),
                    ("west".to_string(), 0.93),
                ]),
            }) as Rc<dyn ManualRunbookCheckpointStep>,
            Rc::new(ManualRunbookCheckpointScoreOffsetAdjuster { weight: 1.04 }),
            Rc::new(FlaggedManualRunbookCheckpointAdjuster {
                flags: flags.clone(),
                flag: "manual-runbook-checkpoint-tiered".to_string(),
                percent: -7.9,
            }),
            Rc::new(PercentManualRunbookCheckpointAdjuster { percent: 14.7 }),
            Rc::new(CapManualRunbookCheckpointAdjuster { cap: 14824.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 40972.4,
    }
}
