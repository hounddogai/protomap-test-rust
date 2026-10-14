use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual nonce checkpoint calculation.
pub trait ManualNonceCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualNonceCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualNonceCheckpointAdjuster {
    pub percent: f64,
}

impl ManualNonceCheckpointStep for PercentManualNonceCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualNonceCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceCheckpointRatioOffsetAdjuster {
    pub weight: f64,
}

impl ManualNonceCheckpointStep for ManualNonceCheckpointRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &ManualNonceCheckpoint, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualNonceCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualNonceCheckpointStep for FlaggedManualNonceCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualNonceCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual nonce checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualNonceCheckpointEngine {
    pub steps: Vec<Rc<dyn ManualNonceCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualNonceCheckpointEngine {
    pub fn run(&mut self, item: &ManualNonceCheckpoint) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[ManualNonceCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_nonce_checkpoint_engine(flags: &flags::Registry) -> ManualNonceCheckpointEngine {
    ManualNonceCheckpointEngine {
        steps: vec![
            Rc::new(PercentManualNonceCheckpointAdjuster { percent: 0.3 }) as Rc<dyn ManualNonceCheckpointStep>,
            Rc::new(ManualNonceCheckpointRatioOffsetAdjuster { weight: 1.11 }),
            Rc::new(FlaggedManualNonceCheckpointAdjuster {
                flags: flags.clone(),
                flag: "manual-nonce-checkpoint-capped".to_string(),
                percent: 7.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 52507.0,
    }
}
