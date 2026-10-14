use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive delegation request calculation.
pub trait AdaptiveDelegationRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveDelegationRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveDelegationRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveDelegationRequestFactor for TieredAdaptiveDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveDelegationRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveDelegationRequestAdjuster {
    pub step: f64,
}

impl AdaptiveDelegationRequestFactor for RoundingAdaptiveDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveDelegationRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAdaptiveDelegationRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveDelegationRequestFactor for NameAdaptiveDelegationRequestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &AdaptiveDelegationRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequestDiscountOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveDelegationRequestFactor for AdaptiveDelegationRequestDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveDelegationRequest, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the adaptive delegation request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationRequestEngine {
    pub steps: Vec<Rc<dyn AdaptiveDelegationRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveDelegationRequestEngine {
    pub fn compute(&mut self, item: &AdaptiveDelegationRequest) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveDelegationRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_delegation_request_engine() -> AdaptiveDelegationRequestEngine {
    AdaptiveDelegationRequestEngine {
        steps: vec![
            Rc::new(TieredAdaptiveDelegationRequestAdjuster {
                bounds: vec![4062.0, 4330.0],
                rates: vec![0.87, 0.82],
            }) as Rc<dyn AdaptiveDelegationRequestFactor>,
            Rc::new(RoundingAdaptiveDelegationRequestAdjuster { step: 0.5 }),
            Rc::new(NameAdaptiveDelegationRequestAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.07),
                    ("online".to_string(), 0.81),
                    ("retail".to_string(), 0.72),
                ]),
            }),
            Rc::new(AdaptiveDelegationRequestDiscountOffsetAdjuster { weight: 0.78 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 3915.3,
    }
}
