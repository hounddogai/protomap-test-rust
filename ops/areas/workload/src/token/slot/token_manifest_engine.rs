use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token manifest calculation.
pub trait TokenManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenManifestFactor for TieredTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenManifestAdjuster {
    pub step: f64,
}

impl TokenManifestFactor for RoundingTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenManifestAdjuster {
    pub minimum: f64,
}

impl TokenManifestFactor for MinimumTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTokenManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenManifestFactor for SourceTokenManifestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TokenManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the token manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenManifestEngine {
    pub steps: Vec<Rc<dyn TokenManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenManifestEngine {
    pub fn price(&mut self, item: &TokenManifest) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_token_manifest_engine() -> TokenManifestEngine {
    TokenManifestEngine {
        steps: vec![
            Rc::new(TieredTokenManifestAdjuster {
                bounds: vec![3410.0, 6473.0, 7898.0, 8983.0],
                rates: vec![1.12, 1.14, 0.98, 0.95],
            }) as Rc<dyn TokenManifestFactor>,
            Rc::new(RoundingTokenManifestAdjuster { step: 0.05 }),
            Rc::new(MinimumTokenManifestAdjuster { minimum: 29.5 }),
            Rc::new(SourceTokenManifestAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.84)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 61661.0,
    }
}
