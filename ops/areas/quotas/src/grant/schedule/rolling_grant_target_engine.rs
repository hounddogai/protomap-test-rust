use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling grant target calculation.
pub trait RollingGrantTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingGrantTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantTargetExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl RollingGrantTargetAdjuster for RollingGrantTargetExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &RollingGrantTarget, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingGrantTargetAdjuster {
    pub minimum: f64,
}

impl RollingGrantTargetAdjuster for MinimumRollingGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingGrantTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingGrantTargetAdjuster {
    pub step: f64,
}

impl RollingGrantTargetAdjuster for RoundingRollingGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingGrantTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingGrantTargetAdjuster {
    pub cap: f64,
}

impl RollingGrantTargetAdjuster for CapRollingGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingGrantTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingGrantTargetAdjuster {
    pub percent: f64,
}

impl RollingGrantTargetAdjuster for PercentRollingGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingGrantTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingGrantTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingGrantTargetAdjuster for FlaggedRollingGrantTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingGrantTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling grant target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingGrantTargetEngine {
    pub steps: Vec<Rc<dyn RollingGrantTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingGrantTargetEngine {
    pub fn evaluate(&mut self, item: &RollingGrantTarget) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingGrantTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingGrantTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_grant_target_engine(flags: &flags::Registry) -> RollingGrantTargetEngine {
    RollingGrantTargetEngine {
        steps: vec![
            Rc::new(RollingGrantTargetExpiresAtOffsetAdjuster { weight: 0.25 }) as Rc<dyn RollingGrantTargetAdjuster>,
            Rc::new(MinimumRollingGrantTargetAdjuster { minimum: 40.1 }),
            Rc::new(RoundingRollingGrantTargetAdjuster { step: 1.0 }),
            Rc::new(CapRollingGrantTargetAdjuster { cap: 2999.0 }),
            Rc::new(PercentRollingGrantTargetAdjuster { percent: -4.3 }),
            Rc::new(FlaggedRollingGrantTargetAdjuster {
                flags: flags.clone(),
                flag: "rolling-grant-target-adaptive".to_string(),
                percent: -4.4,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 96304.0,
    }
}
