use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered silence target calculation.
pub trait TieredSilenceTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredSilenceTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredSilenceTargetAdjuster {
    pub step: f64,
}

impl TieredSilenceTargetAdjuster for RoundingTieredSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredSilenceTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredSilenceTargetAdjuster {
    pub minimum: f64,
}

impl TieredSilenceTargetAdjuster for MinimumTieredSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredSilenceTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredSilenceTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredSilenceTargetAdjuster for TieredTieredSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredSilenceTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered silence target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredSilenceTargetEngine {
    pub steps: Vec<Rc<dyn TieredSilenceTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredSilenceTargetEngine {
    pub fn compute(&mut self, item: &TieredSilenceTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredSilenceTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredSilenceTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_silence_target_engine() -> TieredSilenceTargetEngine {
    TieredSilenceTargetEngine {
        steps: vec![
            Rc::new(RoundingTieredSilenceTargetAdjuster { step: 0.05 }) as Rc<dyn TieredSilenceTargetAdjuster>,
            Rc::new(MinimumTieredSilenceTargetAdjuster { minimum: 40.0 }),
            Rc::new(TieredTieredSilenceTargetAdjuster {
                bounds: vec![663.0, 3096.0, 5645.0, 6385.0],
                rates: vec![0.96, 0.83, 1.13, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 47646.6,
    }
}
