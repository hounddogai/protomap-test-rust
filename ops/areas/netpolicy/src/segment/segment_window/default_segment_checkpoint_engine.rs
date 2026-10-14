use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default segment checkpoint calculation.
pub trait DefaultSegmentCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultSegmentCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultSegmentCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultSegmentCheckpointStep for TieredDefaultSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultSegmentCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSegmentCheckpointRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl DefaultSegmentCheckpointStep for DefaultSegmentCheckpointRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultSegmentCheckpoint, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultSegmentCheckpointAdjuster {
    pub percent: f64,
}

impl DefaultSegmentCheckpointStep for PercentDefaultSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultSegmentCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultSegmentCheckpointAdjuster {
    pub step: f64,
}

impl DefaultSegmentCheckpointStep for RoundingDefaultSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultSegmentCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultSegmentCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultSegmentCheckpointStep for FlaggedDefaultSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultSegmentCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultSegmentCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultSegmentCheckpointStep for LabelDefaultSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultSegmentCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default segment checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultSegmentCheckpointEngine {
    pub steps: Vec<Rc<dyn DefaultSegmentCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultSegmentCheckpointEngine {
    pub fn compute(&mut self, item: &DefaultSegmentCheckpoint) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultSegmentCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_segment_checkpoint_engine(flags: &flags::Registry) -> DefaultSegmentCheckpointEngine {
    DefaultSegmentCheckpointEngine {
        steps: vec![
            Rc::new(TieredDefaultSegmentCheckpointAdjuster {
                bounds: vec![109.0, 2403.0],
                rates: vec![1.11, 1.06],
            }) as Rc<dyn DefaultSegmentCheckpointStep>,
            Rc::new(DefaultSegmentCheckpointRotationDaysOffsetAdjuster { weight: 1.27 }),
            Rc::new(PercentDefaultSegmentCheckpointAdjuster { percent: -14.6 }),
            Rc::new(RoundingDefaultSegmentCheckpointAdjuster { step: 1.0 }),
            Rc::new(FlaggedDefaultSegmentCheckpointAdjuster {
                flags: flags.clone(),
                flag: "default-segment-checkpoint-adaptive".to_string(),
                percent: -8.6,
            }),
            Rc::new(LabelDefaultSegmentCheckpointAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.31),
                    ("south".to_string(), 1.25),
                    ("north".to_string(), 1.04),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 5558.9,
    }
}
