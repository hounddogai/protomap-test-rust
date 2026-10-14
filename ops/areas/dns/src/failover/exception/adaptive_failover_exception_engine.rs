use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive failover exception calculation.
pub trait AdaptiveFailoverExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveFailoverException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceAdaptiveFailoverExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveFailoverExceptionFactor for NamespaceAdaptiveFailoverExceptionAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &AdaptiveFailoverException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveFailoverExceptionAdjuster {
    pub step: f64,
}

impl AdaptiveFailoverExceptionFactor for RoundingAdaptiveFailoverExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveFailoverException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveFailoverExceptionAdjuster {
    pub cap: f64,
}

impl AdaptiveFailoverExceptionFactor for CapAdaptiveFailoverExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveFailoverException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFailoverExceptionSequenceOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveFailoverExceptionFactor for AdaptiveFailoverExceptionSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveFailoverException, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the adaptive failover exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveFailoverExceptionEngine {
    pub steps: Vec<Rc<dyn AdaptiveFailoverExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveFailoverExceptionEngine {
    pub fn run(&mut self, item: &AdaptiveFailoverException) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveFailoverExceptionFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveFailoverException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_failover_exception_engine() -> AdaptiveFailoverExceptionEngine {
    AdaptiveFailoverExceptionEngine {
        steps: vec![
            Rc::new(NamespaceAdaptiveFailoverExceptionAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.95), ("retail".to_string(), 1.13)]),
            }) as Rc<dyn AdaptiveFailoverExceptionFactor>,
            Rc::new(RoundingAdaptiveFailoverExceptionAdjuster { step: 0.5 }),
            Rc::new(CapAdaptiveFailoverExceptionAdjuster { cap: 10360.0 }),
            Rc::new(AdaptiveFailoverExceptionSequenceOffsetAdjuster { weight: 0.8 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 88737.9,
    }
}
