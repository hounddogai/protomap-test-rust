use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering claim calculation.
pub trait PeeringClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringClaimStep for TieredPeeringClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaimBudgetOffsetAdjuster {
    pub weight: f64,
}

impl PeeringClaimStep for PeeringClaimBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringClaim, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringClaimAdjuster {
    pub cap: f64,
}

impl PeeringClaimStep for CapPeeringClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the peering claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringClaimEngine {
    pub steps: Vec<Rc<dyn PeeringClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringClaimEngine {
    pub fn price(&mut self, item: &PeeringClaim) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[PeeringClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_peering_claim_engine() -> PeeringClaimEngine {
    PeeringClaimEngine {
        steps: vec![
            Rc::new(TieredPeeringClaimAdjuster {
                bounds: vec![4095.0, 6270.0],
                rates: vec![1.0, 1.05],
            }) as Rc<dyn PeeringClaimStep>,
            Rc::new(PeeringClaimBudgetOffsetAdjuster { weight: 0.94 }),
            Rc::new(CapPeeringClaimAdjuster { cap: 43234.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 87541.6,
    }
}
