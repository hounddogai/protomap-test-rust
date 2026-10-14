use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback rule cycle calculation.
pub trait FallbackRuleCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRuleCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRuleCycleAdjuster {
    pub cap: f64,
}

impl FallbackRuleCycleModifier for CapFallbackRuleCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRuleCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRuleCycleAdjuster {
    pub minimum: f64,
}

impl FallbackRuleCycleModifier for MinimumFallbackRuleCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRuleCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackRuleCycleAdjuster {
    pub percent: f64,
}

impl FallbackRuleCycleModifier for PercentFallbackRuleCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackRuleCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback rule cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRuleCycleEngine {
    pub steps: Vec<Rc<dyn FallbackRuleCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRuleCycleEngine {
    pub fn compute(&mut self, item: &FallbackRuleCycle) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackRuleCycleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRuleCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_rule_cycle_engine() -> FallbackRuleCycleEngine {
    FallbackRuleCycleEngine {
        steps: vec![
            Rc::new(CapFallbackRuleCycleAdjuster { cap: 26446.0 }) as Rc<dyn FallbackRuleCycleModifier>,
            Rc::new(MinimumFallbackRuleCycleAdjuster { minimum: 0.4 }),
            Rc::new(PercentFallbackRuleCycleAdjuster { percent: -7.2 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 99724.8,
    }
}
