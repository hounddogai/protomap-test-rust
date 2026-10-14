use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation variance calculation.
pub trait FederationVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationVarianceModifier for TieredFederationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationVarianceCeilingOffsetAdjuster {
    pub weight: f64,
}

impl FederationVarianceModifier for FederationVarianceCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &FederationVariance, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationVarianceAdjuster {
    pub cap: f64,
}

impl FederationVarianceModifier for CapFederationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationVarianceAdjuster {
    pub step: f64,
}

impl FederationVarianceModifier for RoundingFederationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFederationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationVarianceModifier for ClusterFederationVarianceAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FederationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the federation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationVarianceEngine {
    pub steps: Vec<Rc<dyn FederationVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationVarianceEngine {
    pub fn run(&mut self, item: &FederationVariance) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[FederationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_federation_variance_engine() -> FederationVarianceEngine {
    FederationVarianceEngine {
        steps: vec![
            Rc::new(TieredFederationVarianceAdjuster {
                bounds: vec![536.0, 4634.0, 7168.0],
                rates: vec![1.06, 1.18, 1.2],
            }) as Rc<dyn FederationVarianceModifier>,
            Rc::new(FederationVarianceCeilingOffsetAdjuster { weight: 1.63 }),
            Rc::new(CapFederationVarianceAdjuster { cap: 29372.0 }),
            Rc::new(RoundingFederationVarianceAdjuster { step: 0.5 }),
            Rc::new(ClusterFederationVarianceAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.91), ("retail".to_string(), 0.94)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 96058.4,
    }
}
