use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling region pin plan calculation.
pub trait RollingRegionPinPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRegionPinPlan, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingRegionPinPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingRegionPinPlanAdjuster for FlaggedRollingRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingRegionPinPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanThresholdOffsetAdjuster {
    pub weight: f64,
}

impl RollingRegionPinPlanAdjuster for RollingRegionPinPlanThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &RollingRegionPinPlan, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingRegionPinPlanAdjuster {
    pub minimum: f64,
}

impl RollingRegionPinPlanAdjuster for MinimumRollingRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingRegionPinPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingRegionPinPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingRegionPinPlanAdjuster for TieredRollingRegionPinPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingRegionPinPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling region pin plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRegionPinPlanEngine {
    pub steps: Vec<Rc<dyn RollingRegionPinPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRegionPinPlanEngine {
    pub fn compute(&mut self, item: &RollingRegionPinPlan) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingRegionPinPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_region_pin_plan_engine(flags: &flags::Registry) -> RollingRegionPinPlanEngine {
    RollingRegionPinPlanEngine {
        steps: vec![
            Rc::new(FlaggedRollingRegionPinPlanAdjuster {
                flags: flags.clone(),
                flag: "rolling-region-pin-plan-adaptive".to_string(),
                percent: -9.0,
            }) as Rc<dyn RollingRegionPinPlanAdjuster>,
            Rc::new(RollingRegionPinPlanThresholdOffsetAdjuster { weight: 1.34 }),
            Rc::new(MinimumRollingRegionPinPlanAdjuster { minimum: 49.7 }),
            Rc::new(TieredRollingRegionPinPlanAdjuster {
                bounds: vec![2375.0, 6183.0, 7565.0],
                rates: vec![1.04, 0.95, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 25817.9,
    }
}
