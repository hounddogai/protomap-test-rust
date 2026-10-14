use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout cycle calculation.
pub trait RolloutCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutCycleAdjuster {
    pub step: f64,
}

impl RolloutCycleFactor for RoundingRolloutCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutCycleFactor for FlaggedRolloutCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRolloutCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutCycleFactor for SegmentRolloutCycleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RolloutCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCycleLimitOffsetAdjuster {
    pub weight: f64,
}

impl RolloutCycleFactor for RolloutCycleLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutCycle, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutCycleAdjuster {
    pub percent: f64,
}

impl RolloutCycleFactor for PercentRolloutCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollout cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutCycleEngine {
    pub steps: Vec<Rc<dyn RolloutCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutCycleEngine {
    pub fn compute(&mut self, item: &RolloutCycle) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutCycleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollout_cycle_engine(flags: &flags::Registry) -> RolloutCycleEngine {
    RolloutCycleEngine {
        steps: vec![
            Rc::new(RoundingRolloutCycleAdjuster { step: 0.5 }) as Rc<dyn RolloutCycleFactor>,
            Rc::new(FlaggedRolloutCycleAdjuster {
                flags: flags.clone(),
                flag: "rollout-cycle-regional".to_string(),
                percent: -2.6,
            }),
            Rc::new(SegmentRolloutCycleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.34), ("west".to_string(), 1.06)]),
            }),
            Rc::new(RolloutCycleLimitOffsetAdjuster { weight: 1.0 }),
            Rc::new(PercentRolloutCycleAdjuster { percent: 1.6 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 16535.0,
    }
}
