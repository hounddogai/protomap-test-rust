use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive release tier calculation.
pub trait AdaptiveReleaseTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveReleaseTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveReleaseTierAdjuster {
    pub cap: f64,
}

impl AdaptiveReleaseTierModifier for CapAdaptiveReleaseTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveReleaseTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveReleaseTierAdjuster {
    pub percent: f64,
}

impl AdaptiveReleaseTierModifier for PercentAdaptiveReleaseTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveReleaseTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveReleaseTierAdjuster {
    pub step: f64,
}

impl AdaptiveReleaseTierModifier for RoundingAdaptiveReleaseTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveReleaseTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveReleaseTierAdjuster {
    pub minimum: f64,
}

impl AdaptiveReleaseTierModifier for MinimumAdaptiveReleaseTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveReleaseTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveReleaseTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveReleaseTierModifier for TieredAdaptiveReleaseTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveReleaseTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive release tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveReleaseTierEngine {
    pub steps: Vec<Rc<dyn AdaptiveReleaseTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveReleaseTierEngine {
    pub fn price(&mut self, item: &AdaptiveReleaseTier) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveReleaseTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_release_tier_engine() -> AdaptiveReleaseTierEngine {
    AdaptiveReleaseTierEngine {
        steps: vec![
            Rc::new(CapAdaptiveReleaseTierAdjuster { cap: 10239.0 }) as Rc<dyn AdaptiveReleaseTierModifier>,
            Rc::new(PercentAdaptiveReleaseTierAdjuster { percent: -20.7 }),
            Rc::new(RoundingAdaptiveReleaseTierAdjuster { step: 0.1 }),
            Rc::new(MinimumAdaptiveReleaseTierAdjuster { minimum: 4.8 }),
            Rc::new(TieredAdaptiveReleaseTierAdjuster {
                bounds: vec![3293.0, 6785.0],
                rates: vec![0.97, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 50536.8,
    }
}
