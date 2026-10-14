use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered silence policy calculation.
pub trait TieredSilencePolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredSilencePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePolicyDiscountOffsetAdjuster {
    pub weight: f64,
}

impl TieredSilencePolicyAdjuster for TieredSilencePolicyDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &TieredSilencePolicy, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredSilencePolicyAdjuster {
    pub minimum: f64,
}

impl TieredSilencePolicyAdjuster for MinimumTieredSilencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredSilencePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredSilencePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredSilencePolicyAdjuster for TieredTieredSilencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredSilencePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredSilencePolicyAdjuster {
    pub step: f64,
}

impl TieredSilencePolicyAdjuster for RoundingTieredSilencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredSilencePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered silence policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredSilencePolicyEngine {
    pub steps: Vec<Rc<dyn TieredSilencePolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredSilencePolicyEngine {
    pub fn evaluate(&mut self, item: &TieredSilencePolicy) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[TieredSilencePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_silence_policy_engine() -> TieredSilencePolicyEngine {
    TieredSilencePolicyEngine {
        steps: vec![
            Rc::new(TieredSilencePolicyDiscountOffsetAdjuster { weight: 0.91 }) as Rc<dyn TieredSilencePolicyAdjuster>,
            Rc::new(MinimumTieredSilencePolicyAdjuster { minimum: 13.3 }),
            Rc::new(TieredTieredSilencePolicyAdjuster {
                bounds: vec![1089.0, 7769.0],
                rates: vec![1.19, 1.08],
            }),
            Rc::new(RoundingTieredSilencePolicyAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 34137.5,
    }
}
