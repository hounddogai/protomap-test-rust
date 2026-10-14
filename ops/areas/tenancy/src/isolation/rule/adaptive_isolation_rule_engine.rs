use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive isolation rule calculation.
pub trait AdaptiveIsolationRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveIsolationRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveIsolationRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveIsolationRuleModifier for TieredAdaptiveIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveIsolationRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationRuleBurstOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveIsolationRuleModifier for AdaptiveIsolationRuleBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveIsolationRule, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveIsolationRuleAdjuster {
    pub percent: f64,
}

impl AdaptiveIsolationRuleModifier for PercentAdaptiveIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveIsolationRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive isolation rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveIsolationRuleEngine {
    pub steps: Vec<Rc<dyn AdaptiveIsolationRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveIsolationRuleEngine {
    pub fn run(&mut self, item: &AdaptiveIsolationRule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveIsolationRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveIsolationRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_isolation_rule_engine() -> AdaptiveIsolationRuleEngine {
    AdaptiveIsolationRuleEngine {
        steps: vec![
            Rc::new(TieredAdaptiveIsolationRuleAdjuster {
                bounds: vec![7744.0, 8803.0],
                rates: vec![1.11, 1.13],
            }) as Rc<dyn AdaptiveIsolationRuleModifier>,
            Rc::new(AdaptiveIsolationRuleBurstOffsetAdjuster { weight: 1.13 }),
            Rc::new(PercentAdaptiveIsolationRuleAdjuster { percent: -21.8 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 55190.0,
    }
}
