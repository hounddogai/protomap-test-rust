use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary policy item calculation.
pub trait PrimaryPolicyItemFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryPolicyItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryPolicyItemAdjuster {
    pub cap: f64,
}

impl PrimaryPolicyItemFactor for CapPrimaryPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryPolicyItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryPolicyItemAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryPolicyItemFactor for SourcePrimaryPolicyItemAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryPolicyItem, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryPolicyItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryPolicyItemFactor for TieredPrimaryPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryPolicyItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryPolicyItemAdjuster {
    pub step: f64,
}

impl PrimaryPolicyItemFactor for RoundingPrimaryPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryPolicyItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary policy item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryPolicyItemEngine {
    pub steps: Vec<Rc<dyn PrimaryPolicyItemFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryPolicyItemEngine {
    pub fn price(&mut self, item: &PrimaryPolicyItem) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryPolicyItemFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryPolicyItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_policy_item_engine() -> PrimaryPolicyItemEngine {
    PrimaryPolicyItemEngine {
        steps: vec![
            Rc::new(CapPrimaryPolicyItemAdjuster { cap: 36909.0 }) as Rc<dyn PrimaryPolicyItemFactor>,
            Rc::new(SourcePrimaryPolicyItemAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.01)]),
            }),
            Rc::new(TieredPrimaryPolicyItemAdjuster {
                bounds: vec![3124.0, 5771.0, 8134.0],
                rates: vec![0.81, 0.87, 1.14],
            }),
            Rc::new(RoundingPrimaryPolicyItemAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 91102.2,
    }
}
