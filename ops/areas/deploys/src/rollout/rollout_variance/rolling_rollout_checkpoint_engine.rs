use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling rollout checkpoint calculation.
pub trait RollingRolloutCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRolloutCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingRolloutCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingRolloutCheckpointStep for TieredRollingRolloutCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingRolloutCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingRolloutCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingRolloutCheckpointStep for FlaggedRollingRolloutCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingRolloutCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingRolloutCheckpointAdjuster {
    pub step: f64,
}

impl RollingRolloutCheckpointStep for RoundingRollingRolloutCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingRolloutCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingRolloutCheckpointAdjuster {
    pub minimum: f64,
}

impl RollingRolloutCheckpointStep for MinimumRollingRolloutCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingRolloutCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingRolloutCheckpointAdjuster {
    pub percent: f64,
}

impl RollingRolloutCheckpointStep for PercentRollingRolloutCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingRolloutCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutCheckpointMarginOffsetAdjuster {
    pub weight: f64,
}

impl RollingRolloutCheckpointStep for RollingRolloutCheckpointMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RollingRolloutCheckpoint, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the rolling rollout checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRolloutCheckpointEngine {
    pub steps: Vec<Rc<dyn RollingRolloutCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRolloutCheckpointEngine {
    pub fn evaluate(&mut self, item: &RollingRolloutCheckpoint) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingRolloutCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_rollout_checkpoint_engine(flags: &flags::Registry) -> RollingRolloutCheckpointEngine {
    RollingRolloutCheckpointEngine {
        steps: vec![
            Rc::new(TieredRollingRolloutCheckpointAdjuster {
                bounds: vec![2040.0, 3666.0, 7164.0],
                rates: vec![1.01, 0.96, 0.93],
            }) as Rc<dyn RollingRolloutCheckpointStep>,
            Rc::new(FlaggedRollingRolloutCheckpointAdjuster {
                flags: flags.clone(),
                flag: "rolling-rollout-checkpoint-batched".to_string(),
                percent: 2.1,
            }),
            Rc::new(RoundingRollingRolloutCheckpointAdjuster { step: 0.1 }),
            Rc::new(MinimumRollingRolloutCheckpointAdjuster { minimum: 9.8 }),
            Rc::new(PercentRollingRolloutCheckpointAdjuster { percent: 20.1 }),
            Rc::new(RollingRolloutCheckpointMarginOffsetAdjuster { weight: 1.14 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 83143.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_rollout_checkpoint_engine(&flags::new_registry());
        let item = new_rolling_rollout_checkpoint("code-260", "label-525");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
