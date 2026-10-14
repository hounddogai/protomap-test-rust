use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rolling slo target calculation.
pub trait RollingSloTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingSloTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingSloTargetAdjuster {
    pub percent: f64,
}

impl RollingSloTargetFactor for PercentRollingSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingSloTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRollingSloTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingSloTargetFactor for ClusterRollingSloTargetAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RollingSloTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingSloTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingSloTargetFactor for FlaggedRollingSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingSloTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingSloTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingSloTargetFactor for TieredRollingSloTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingSloTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling slo target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingSloTargetEngine {
    pub steps: Vec<Rc<dyn RollingSloTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingSloTargetEngine {
    pub fn evaluate(&mut self, item: &RollingSloTarget) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingSloTargetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingSloTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_slo_target_engine(flags: &flags::Registry) -> RollingSloTargetEngine {
    RollingSloTargetEngine {
        steps: vec![
            Rc::new(PercentRollingSloTargetAdjuster { percent: -2.6 }) as Rc<dyn RollingSloTargetFactor>,
            Rc::new(ClusterRollingSloTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.36), ("online".to_string(), 0.91)]),
            }),
            Rc::new(FlaggedRollingSloTargetAdjuster {
                flags: flags.clone(),
                flag: "rolling-slo-target-rolling".to_string(),
                percent: 4.6,
            }),
            Rc::new(TieredRollingSloTargetAdjuster {
                bounds: vec![1743.0, 7387.0, 9892.0],
                rates: vec![0.88, 1.04, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 49406.1,
    }
}
