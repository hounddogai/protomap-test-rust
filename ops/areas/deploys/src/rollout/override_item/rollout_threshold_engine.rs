use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout threshold calculation.
pub trait RolloutThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutThresholdAdjuster {
    pub percent: f64,
}

impl RolloutThresholdStep for PercentRolloutThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutThresholdAdjuster {
    pub step: f64,
}

impl RolloutThresholdStep for RoundingRolloutThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutThresholdAdjuster {
    pub minimum: f64,
}

impl RolloutThresholdStep for MinimumRolloutThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutThresholdStep for FlaggedRolloutThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollout threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutThresholdEngine {
    pub steps: Vec<Rc<dyn RolloutThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutThresholdEngine {
    pub fn evaluate(&mut self, item: &RolloutThreshold) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_threshold_engine(flags: &flags::Registry) -> RolloutThresholdEngine {
    RolloutThresholdEngine {
        steps: vec![
            Rc::new(PercentRolloutThresholdAdjuster { percent: -21.4 }) as Rc<dyn RolloutThresholdStep>,
            Rc::new(RoundingRolloutThresholdAdjuster { step: 5.0 }),
            Rc::new(MinimumRolloutThresholdAdjuster { minimum: 39.9 }),
            Rc::new(FlaggedRolloutThresholdAdjuster {
                flags: flags.clone(),
                flag: "rollout-threshold-seasonal".to_string(),
                percent: 3.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 82933.4,
    }
}
