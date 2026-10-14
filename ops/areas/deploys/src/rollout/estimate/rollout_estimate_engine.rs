use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout estimate calculation.
pub trait RolloutEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutEstimateAdjuster {
    pub step: f64,
}

impl RolloutEstimateStep for RoundingRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutEstimateVersionOffsetAdjuster {
    pub weight: f64,
}

impl RolloutEstimateStep for RolloutEstimateVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutEstimate, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutEstimateAdjuster {
    pub percent: f64,
}

impl RolloutEstimateStep for PercentRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutEstimateStep for FlaggedRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutEstimateAdjuster {
    pub minimum: f64,
}

impl RolloutEstimateStep for MinimumRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRolloutEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutEstimateStep for SourceRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RolloutEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rollout estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutEstimateEngine {
    pub steps: Vec<Rc<dyn RolloutEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutEstimateEngine {
    pub fn evaluate(&mut self, item: &RolloutEstimate) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutEstimateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_estimate_engine(flags: &flags::Registry) -> RolloutEstimateEngine {
    RolloutEstimateEngine {
        steps: vec![
            Rc::new(RoundingRolloutEstimateAdjuster { step: 0.05 }) as Rc<dyn RolloutEstimateStep>,
            Rc::new(RolloutEstimateVersionOffsetAdjuster { weight: 0.59 }),
            Rc::new(PercentRolloutEstimateAdjuster { percent: 8.0 }),
            Rc::new(FlaggedRolloutEstimateAdjuster {
                flags: flags.clone(),
                flag: "rollout-estimate-default".to_string(),
                percent: -6.5,
            }),
            Rc::new(MinimumRolloutEstimateAdjuster { minimum: 18.8 }),
            Rc::new(SourceRolloutEstimateAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.29)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 46957.6,
    }
}
