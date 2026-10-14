use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive measurement rule calculation.
pub trait AdaptiveMeasurementRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveMeasurementRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveMeasurementRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveMeasurementRuleModifier for TieredAdaptiveMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveMeasurementRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveMeasurementRuleAdjuster {
    pub step: f64,
}

impl AdaptiveMeasurementRuleModifier for RoundingAdaptiveMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveMeasurementRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveMeasurementRuleAdjuster {
    pub cap: f64,
}

impl AdaptiveMeasurementRuleModifier for CapAdaptiveMeasurementRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveMeasurementRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive measurement rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveMeasurementRuleEngine {
    pub steps: Vec<Rc<dyn AdaptiveMeasurementRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveMeasurementRuleEngine {
    pub fn run(&mut self, item: &AdaptiveMeasurementRule) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveMeasurementRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveMeasurementRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_measurement_rule_engine() -> AdaptiveMeasurementRuleEngine {
    AdaptiveMeasurementRuleEngine {
        steps: vec![
            Rc::new(TieredAdaptiveMeasurementRuleAdjuster {
                bounds: vec![757.0, 2744.0, 2928.0, 4712.0],
                rates: vec![1.15, 0.87, 1.07, 0.89],
            }) as Rc<dyn AdaptiveMeasurementRuleModifier>,
            Rc::new(RoundingAdaptiveMeasurementRuleAdjuster { step: 0.5 }),
            Rc::new(CapAdaptiveMeasurementRuleAdjuster { cap: 44624.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 6682.6,
    }
}
