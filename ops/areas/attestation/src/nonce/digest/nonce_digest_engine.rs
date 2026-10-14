use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the nonce digest calculation.
pub trait NonceDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceDigestAdjuster {
    pub cap: f64,
}

impl NonceDigestFactor for CapNonceDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceDigestFactor for TieredNonceDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigestQuantityOffsetAdjuster {
    pub weight: f64,
}

impl NonceDigestFactor for NonceDigestQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &NonceDigest, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitNonceDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NonceDigestFactor for UnitNonceDigestAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &NonceDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceDigestAdjuster {
    pub step: f64,
}

impl NonceDigestFactor for RoundingNonceDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceDigestAdjuster {
    pub minimum: f64,
}

impl NonceDigestFactor for MinimumNonceDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the nonce digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceDigestEngine {
    pub steps: Vec<Rc<dyn NonceDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceDigestEngine {
    pub fn evaluate(&mut self, item: &NonceDigest) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceDigestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_digest_engine() -> NonceDigestEngine {
    NonceDigestEngine {
        steps: vec![
            Rc::new(CapNonceDigestAdjuster { cap: 16914.0 }) as Rc<dyn NonceDigestFactor>,
            Rc::new(TieredNonceDigestAdjuster {
                bounds: vec![607.0, 4181.0, 7842.0, 9604.0],
                rates: vec![0.8, 0.98, 1.13, 1.17],
            }),
            Rc::new(NonceDigestQuantityOffsetAdjuster { weight: 1.52 }),
            Rc::new(UnitNonceDigestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.19)]),
            }),
            Rc::new(RoundingNonceDigestAdjuster { step: 0.25 }),
            Rc::new(MinimumNonceDigestAdjuster { minimum: 47.9 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 48526.0,
    }
}
