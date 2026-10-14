use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation window calculation.
pub trait FederationWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationWindowFactor for TieredFederationWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationWindowAdjuster {
    pub step: f64,
}

impl FederationWindowFactor for RoundingFederationWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationWindowAdjuster {
    pub minimum: f64,
}

impl FederationWindowFactor for MinimumFederationWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFederationWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationWindowFactor for ClusterFederationWindowAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FederationWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the federation window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationWindowEngine {
    pub steps: Vec<Rc<dyn FederationWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationWindowEngine {
    pub fn compute(&mut self, item: &FederationWindow) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationWindowFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_federation_window_engine() -> FederationWindowEngine {
    FederationWindowEngine {
        steps: vec![
            Rc::new(TieredFederationWindowAdjuster {
                bounds: vec![3987.0, 5447.0, 5512.0, 9996.0],
                rates: vec![0.82, 1.03, 0.99, 0.89],
            }) as Rc<dyn FederationWindowFactor>,
            Rc::new(RoundingFederationWindowAdjuster { step: 0.25 }),
            Rc::new(MinimumFederationWindowAdjuster { minimum: 16.9 }),
            Rc::new(ClusterFederationWindowAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.39),
                    ("central".to_string(), 0.93),
                    ("east".to_string(), 1.03),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 48498.9,
    }
}
