use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling plan checkpoint calculation.
pub trait RollingPlanCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPlanCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanCheckpointBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl RollingPlanCheckpointModifier for RollingPlanCheckpointBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPlanCheckpoint, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingPlanCheckpointAdjuster {
    pub minimum: f64,
}

impl RollingPlanCheckpointModifier for MinimumRollingPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingPlanCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPlanCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPlanCheckpointModifier for TieredRollingPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPlanCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPlanCheckpointAdjuster {
    pub step: f64,
}

impl RollingPlanCheckpointModifier for RoundingRollingPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPlanCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPlanCheckpointAdjuster {
    pub cap: f64,
}

impl RollingPlanCheckpointModifier for CapRollingPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPlanCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPlanCheckpointAdjuster {
    pub percent: f64,
}

impl RollingPlanCheckpointModifier for PercentRollingPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPlanCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling plan checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPlanCheckpointEngine {
    pub steps: Vec<Rc<dyn RollingPlanCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPlanCheckpointEngine {
    pub fn evaluate(&mut self, item: &RollingPlanCheckpoint) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingPlanCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_plan_checkpoint_engine() -> RollingPlanCheckpointEngine {
    RollingPlanCheckpointEngine {
        steps: vec![
            Rc::new(RollingPlanCheckpointBatchSizeOffsetAdjuster { weight: 1.84 })
                as Rc<dyn RollingPlanCheckpointModifier>,
            Rc::new(MinimumRollingPlanCheckpointAdjuster { minimum: 10.5 }),
            Rc::new(TieredRollingPlanCheckpointAdjuster {
                bounds: vec![6126.0, 8796.0, 9630.0],
                rates: vec![0.96, 0.98, 1.19],
            }),
            Rc::new(RoundingRollingPlanCheckpointAdjuster { step: 0.5 }),
            Rc::new(CapRollingPlanCheckpointAdjuster { cap: 45870.0 }),
            Rc::new(PercentRollingPlanCheckpointAdjuster { percent: 21.5 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 20303.4,
    }
}
