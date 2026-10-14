use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient canary tier calculation.
pub trait LenientCanaryTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientCanaryTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientCanaryTierAdjuster {
    pub step: f64,
}

impl LenientCanaryTierFactor for RoundingLenientCanaryTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientCanaryTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryTierBudgetOffsetAdjuster {
    pub weight: f64,
}

impl LenientCanaryTierFactor for LenientCanaryTierBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &LenientCanaryTier, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientCanaryTierAdjuster {
    pub minimum: f64,
}

impl LenientCanaryTierFactor for MinimumLenientCanaryTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientCanaryTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient canary tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientCanaryTierEngine {
    pub steps: Vec<Rc<dyn LenientCanaryTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientCanaryTierEngine {
    pub fn compute(&mut self, item: &LenientCanaryTier) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientCanaryTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_canary_tier_engine() -> LenientCanaryTierEngine {
    LenientCanaryTierEngine {
        steps: vec![
            Rc::new(RoundingLenientCanaryTierAdjuster { step: 5.0 }) as Rc<dyn LenientCanaryTierFactor>,
            Rc::new(LenientCanaryTierBudgetOffsetAdjuster { weight: 1.18 }),
            Rc::new(MinimumLenientCanaryTierAdjuster { minimum: 32.6 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 76923.5,
    }
}
