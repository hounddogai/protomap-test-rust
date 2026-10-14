use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension target calculation.
pub trait SuspensionTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionTargetAdjuster {
    pub step: f64,
}

impl SuspensionTargetModifier for RoundingSuspensionTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTargetWeightOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionTargetModifier for SuspensionTargetWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionTarget, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionTargetAdjuster {
    pub minimum: f64,
}

impl SuspensionTargetModifier for MinimumSuspensionTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionTargetAdjuster {
    pub percent: f64,
}

impl SuspensionTargetModifier for PercentSuspensionTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the suspension target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionTargetEngine {
    pub steps: Vec<Rc<dyn SuspensionTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionTargetEngine {
    pub fn evaluate(&mut self, item: &SuspensionTarget) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionTargetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_target_engine() -> SuspensionTargetEngine {
    SuspensionTargetEngine {
        steps: vec![
            Rc::new(RoundingSuspensionTargetAdjuster { step: 5.0 }) as Rc<dyn SuspensionTargetModifier>,
            Rc::new(SuspensionTargetWeightOffsetAdjuster { weight: 1.02 }),
            Rc::new(MinimumSuspensionTargetAdjuster { minimum: 46.4 }),
            Rc::new(PercentSuspensionTargetAdjuster { percent: 7.7 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 35442.1,
    }
}
