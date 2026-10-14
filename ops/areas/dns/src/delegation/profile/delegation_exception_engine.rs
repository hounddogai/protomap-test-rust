use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the delegation exception calculation.
pub trait DelegationExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationExceptionFactor for TieredDelegationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationExceptionAdjuster {
    pub step: f64,
}

impl DelegationExceptionFactor for RoundingDelegationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationExceptionAdjuster {
    pub cap: f64,
}

impl DelegationExceptionFactor for CapDelegationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the delegation exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationExceptionEngine {
    pub steps: Vec<Rc<dyn DelegationExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationExceptionEngine {
    pub fn compute(&mut self, item: &DelegationException) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[DelegationException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_exception_engine() -> DelegationExceptionEngine {
    DelegationExceptionEngine {
        steps: vec![
            Rc::new(TieredDelegationExceptionAdjuster {
                bounds: vec![5198.0, 8339.0],
                rates: vec![1.05, 0.83],
            }) as Rc<dyn DelegationExceptionFactor>,
            Rc::new(RoundingDelegationExceptionAdjuster { step: 5.0 }),
            Rc::new(CapDelegationExceptionAdjuster { cap: 20004.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 84072.2,
    }
}
