use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered rollout segment calculation.
pub trait TieredRolloutSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRolloutSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredRolloutSegmentAdjuster {
    pub step: f64,
}

impl TieredRolloutSegmentModifier for RoundingTieredRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredRolloutSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredRolloutSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredRolloutSegmentModifier for FlaggedTieredRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredRolloutSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredRolloutSegmentAdjuster {
    pub minimum: f64,
}

impl TieredRolloutSegmentModifier for MinimumTieredRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredRolloutSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredRolloutSegmentAdjuster {
    pub percent: f64,
}

impl TieredRolloutSegmentModifier for PercentTieredRolloutSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredRolloutSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered rollout segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRolloutSegmentEngine {
    pub steps: Vec<Rc<dyn TieredRolloutSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRolloutSegmentEngine {
    pub fn compute(&mut self, item: &TieredRolloutSegment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredRolloutSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredRolloutSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_rollout_segment_engine(flags: &flags::Registry) -> TieredRolloutSegmentEngine {
    TieredRolloutSegmentEngine {
        steps: vec![
            Rc::new(RoundingTieredRolloutSegmentAdjuster { step: 0.05 }) as Rc<dyn TieredRolloutSegmentModifier>,
            Rc::new(FlaggedTieredRolloutSegmentAdjuster {
                flags: flags.clone(),
                flag: "tiered-rollout-segment-manual".to_string(),
                percent: -4.2,
            }),
            Rc::new(MinimumTieredRolloutSegmentAdjuster { minimum: 34.8 }),
            Rc::new(PercentTieredRolloutSegmentAdjuster { percent: -8.1 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 9741.2,
    }
}
