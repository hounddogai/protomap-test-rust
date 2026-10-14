use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default action item target calculation.
pub trait DefaultActionItemTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultActionItemTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultActionItemTargetAdjuster {
    pub cap: f64,
}

impl DefaultActionItemTargetStep for CapDefaultActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultActionItemTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultActionItemTargetAdjuster {
    pub percent: f64,
}

impl DefaultActionItemTargetStep for PercentDefaultActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultActionItemTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultActionItemTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultActionItemTargetStep for TieredDefaultActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultActionItemTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultActionItemTargetAdjuster {
    pub step: f64,
}

impl DefaultActionItemTargetStep for RoundingDefaultActionItemTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultActionItemTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default action item target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultActionItemTargetEngine {
    pub steps: Vec<Rc<dyn DefaultActionItemTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultActionItemTargetEngine {
    pub fn evaluate(&mut self, item: &DefaultActionItemTarget) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultActionItemTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultActionItemTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_action_item_target_engine() -> DefaultActionItemTargetEngine {
    DefaultActionItemTargetEngine {
        steps: vec![
            Rc::new(CapDefaultActionItemTargetAdjuster { cap: 33801.0 }) as Rc<dyn DefaultActionItemTargetStep>,
            Rc::new(PercentDefaultActionItemTargetAdjuster { percent: 10.0 }),
            Rc::new(TieredDefaultActionItemTargetAdjuster {
                bounds: vec![167.0, 6678.0, 9030.0, 9630.0],
                rates: vec![1.19, 0.85, 0.81, 0.91],
            }),
            Rc::new(RoundingDefaultActionItemTargetAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 78421.7,
    }
}
