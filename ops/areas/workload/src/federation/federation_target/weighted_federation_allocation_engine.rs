use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted federation allocation calculation.
pub trait WeightedFederationAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFederationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationAllocationUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFederationAllocationModifier for WeightedFederationAllocationUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFederationAllocation, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedFederationAllocationAdjuster {
    pub cap: f64,
}

impl WeightedFederationAllocationModifier for CapWeightedFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedFederationAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeWeightedFederationAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedFederationAllocationModifier for ScopeWeightedFederationAllocationAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &WeightedFederationAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted federation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFederationAllocationEngine {
    pub steps: Vec<Rc<dyn WeightedFederationAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFederationAllocationEngine {
    pub fn price(&mut self, item: &WeightedFederationAllocation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedFederationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_federation_allocation_engine() -> WeightedFederationAllocationEngine {
    WeightedFederationAllocationEngine {
        steps: vec![
            Rc::new(WeightedFederationAllocationUpdatedAtOffsetAdjuster { weight: 0.04 })
                as Rc<dyn WeightedFederationAllocationModifier>,
            Rc::new(CapWeightedFederationAllocationAdjuster { cap: 45965.0 }),
            Rc::new(ScopeWeightedFederationAllocationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.75), ("west".to_string(), 1.01)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 1060.4,
    }
}
