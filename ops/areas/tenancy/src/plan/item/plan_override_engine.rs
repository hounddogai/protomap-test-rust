use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan override calculation.
pub trait PlanOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanOverrideAdjuster {
    pub step: f64,
}

impl PlanOverrideModifier for RoundingPlanOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanOverrideAdjuster {
    pub cap: f64,
}

impl PlanOverrideModifier for CapPlanOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanOverrideModifier for TieredPlanOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanOverrideModifier for FlaggedPlanOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanOverrideEngine {
    pub steps: Vec<Rc<dyn PlanOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanOverrideEngine {
    pub fn compute(&mut self, item: &PlanOverride) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_override_engine(flags: &flags::Registry) -> PlanOverrideEngine {
    PlanOverrideEngine {
        steps: vec![
            Rc::new(RoundingPlanOverrideAdjuster { step: 0.5 }) as Rc<dyn PlanOverrideModifier>,
            Rc::new(CapPlanOverrideAdjuster { cap: 30172.0 }),
            Rc::new(TieredPlanOverrideAdjuster {
                bounds: vec![249.0, 8179.0, 8492.0, 9273.0],
                rates: vec![1.13, 0.9, 1.19, 1.0],
            }),
            Rc::new(FlaggedPlanOverrideAdjuster {
                flags: flags.clone(),
                flag: "plan-override-weighted".to_string(),
                percent: -6.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 73898.1,
    }
}
