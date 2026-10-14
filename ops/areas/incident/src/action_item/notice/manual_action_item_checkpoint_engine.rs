use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual action item checkpoint calculation.
pub trait ManualActionItemCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualActionItemCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualActionItemCheckpointAdjuster {
    pub minimum: f64,
}

impl ManualActionItemCheckpointFactor for MinimumManualActionItemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualActionItemCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualActionItemCheckpointAdjuster {
    pub percent: f64,
}

impl ManualActionItemCheckpointFactor for PercentManualActionItemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualActionItemCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualActionItemCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualActionItemCheckpointFactor for FlaggedManualActionItemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualActionItemCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualActionItemCheckpointAdjuster {
    pub step: f64,
}

impl ManualActionItemCheckpointFactor for RoundingManualActionItemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualActionItemCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleManualActionItemCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualActionItemCheckpointFactor for TitleManualActionItemCheckpointAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ManualActionItemCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualActionItemCheckpointAdjuster {
    pub cap: f64,
}

impl ManualActionItemCheckpointFactor for CapManualActionItemCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualActionItemCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual action item checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualActionItemCheckpointEngine {
    pub steps: Vec<Rc<dyn ManualActionItemCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualActionItemCheckpointEngine {
    pub fn evaluate(&mut self, item: &ManualActionItemCheckpoint) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[ManualActionItemCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_action_item_checkpoint_engine(flags: &flags::Registry) -> ManualActionItemCheckpointEngine {
    ManualActionItemCheckpointEngine {
        steps: vec![
            Rc::new(MinimumManualActionItemCheckpointAdjuster { minimum: 1.3 })
                as Rc<dyn ManualActionItemCheckpointFactor>,
            Rc::new(PercentManualActionItemCheckpointAdjuster { percent: 7.2 }),
            Rc::new(FlaggedManualActionItemCheckpointAdjuster {
                flags: flags.clone(),
                flag: "manual-action-item-checkpoint-primary".to_string(),
                percent: 7.5,
            }),
            Rc::new(RoundingManualActionItemCheckpointAdjuster { step: 5.0 }),
            Rc::new(TitleManualActionItemCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.28),
                    ("west".to_string(), 1.18),
                    ("south".to_string(), 1.13),
                ]),
            }),
            Rc::new(CapManualActionItemCheckpointAdjuster { cap: 23828.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 15206.2,
    }
}
