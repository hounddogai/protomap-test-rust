use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout tier calculation.
pub trait RolloutTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutTierAdjuster {
    pub step: f64,
}

impl RolloutTierModifier for RoundingRolloutTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutTierAdjuster {
    pub percent: f64,
}

impl RolloutTierModifier for PercentRolloutTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutTierModifier for FlaggedRolloutTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutTierAdjuster {
    pub minimum: f64,
}

impl RolloutTierModifier for MinimumRolloutTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollout tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutTierEngine {
    pub steps: Vec<Rc<dyn RolloutTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutTierEngine {
    pub fn evaluate(&mut self, item: &RolloutTier) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RolloutTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_tier_engine(flags: &flags::Registry) -> RolloutTierEngine {
    RolloutTierEngine {
        steps: vec![
            Rc::new(RoundingRolloutTierAdjuster { step: 0.5 }) as Rc<dyn RolloutTierModifier>,
            Rc::new(PercentRolloutTierAdjuster { percent: -21.9 }),
            Rc::new(FlaggedRolloutTierAdjuster {
                flags: flags.clone(),
                flag: "rollout-tier-seasonal".to_string(),
                percent: -1.6,
            }),
            Rc::new(MinimumRolloutTierAdjuster { minimum: 49.1 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 74834.9,
    }
}
