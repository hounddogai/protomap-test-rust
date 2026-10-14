use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress manifest calculation.
pub trait EgressManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressManifestAdjuster {
    pub percent: f64,
}

impl EgressManifestFactor for PercentEgressManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressManifestFactor for TieredEgressManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressManifestBudgetOffsetAdjuster {
    pub weight: f64,
}

impl EgressManifestFactor for EgressManifestBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &EgressManifest, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the egress manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressManifestEngine {
    pub steps: Vec<Rc<dyn EgressManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressManifestEngine {
    pub fn price(&mut self, item: &EgressManifest) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[EgressManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_egress_manifest_engine() -> EgressManifestEngine {
    EgressManifestEngine {
        steps: vec![
            Rc::new(PercentEgressManifestAdjuster { percent: 2.2 }) as Rc<dyn EgressManifestFactor>,
            Rc::new(TieredEgressManifestAdjuster {
                bounds: vec![2244.0, 5502.0],
                rates: vec![1.07, 1.19],
            }),
            Rc::new(EgressManifestBudgetOffsetAdjuster { weight: 0.78 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 87332.4,
    }
}
