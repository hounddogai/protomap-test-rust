use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled rollout rollup calculation.
pub trait ScheduledRolloutRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRolloutRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeScheduledRolloutRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledRolloutRollupStep for ScopeScheduledRolloutRollupAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ScheduledRolloutRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledRolloutRollupAdjuster {
    pub step: f64,
}

impl ScheduledRolloutRollupStep for RoundingScheduledRolloutRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledRolloutRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledRolloutRollupAdjuster {
    pub minimum: f64,
}

impl ScheduledRolloutRollupStep for MinimumScheduledRolloutRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledRolloutRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutRollupPriorityOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledRolloutRollupStep for ScheduledRolloutRollupPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledRolloutRollup, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledRolloutRollupAdjuster {
    pub percent: f64,
}

impl ScheduledRolloutRollupStep for PercentScheduledRolloutRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledRolloutRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledRolloutRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledRolloutRollupStep for FlaggedScheduledRolloutRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledRolloutRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled rollout rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRolloutRollupEngine {
    pub steps: Vec<Rc<dyn ScheduledRolloutRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRolloutRollupEngine {
    pub fn price(&mut self, item: &ScheduledRolloutRollup) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledRolloutRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_rollout_rollup_engine(flags: &flags::Registry) -> ScheduledRolloutRollupEngine {
    ScheduledRolloutRollupEngine {
        steps: vec![
            Rc::new(ScopeScheduledRolloutRollupAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.09),
                    ("south".to_string(), 1.34),
                    ("online".to_string(), 1.04),
                ]),
            }) as Rc<dyn ScheduledRolloutRollupStep>,
            Rc::new(RoundingScheduledRolloutRollupAdjuster { step: 5.0 }),
            Rc::new(MinimumScheduledRolloutRollupAdjuster { minimum: 46.0 }),
            Rc::new(ScheduledRolloutRollupPriorityOffsetAdjuster { weight: 0.97 }),
            Rc::new(PercentScheduledRolloutRollupAdjuster { percent: -21.1 }),
            Rc::new(FlaggedScheduledRolloutRollupAdjuster {
                flags: flags.clone(),
                flag: "scheduled-rollout-rollup-regional".to_string(),
                percent: -7.4,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 26922.4,
    }
}
