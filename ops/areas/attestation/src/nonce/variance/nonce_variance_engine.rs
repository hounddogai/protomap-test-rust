use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the nonce variance calculation.
pub trait NonceVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceVarianceQuantityOffsetAdjuster {
    pub weight: f64,
}

impl NonceVarianceAdjuster for NonceVarianceQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &NonceVariance, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceVarianceAdjuster for TieredNonceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceVarianceAdjuster for FlaggedNonceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceVarianceAdjuster {
    pub minimum: f64,
}

impl NonceVarianceAdjuster for MinimumNonceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the nonce variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceVarianceEngine {
    pub steps: Vec<Rc<dyn NonceVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceVarianceEngine {
    pub fn evaluate(&mut self, item: &NonceVariance) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceVarianceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_variance_engine(flags: &flags::Registry) -> NonceVarianceEngine {
    NonceVarianceEngine {
        steps: vec![
            Rc::new(NonceVarianceQuantityOffsetAdjuster { weight: 0.62 }) as Rc<dyn NonceVarianceAdjuster>,
            Rc::new(TieredNonceVarianceAdjuster {
                bounds: vec![396.0, 3457.0, 4223.0],
                rates: vec![1.0, 1.06, 1.07],
            }),
            Rc::new(FlaggedNonceVarianceAdjuster {
                flags: flags.clone(),
                flag: "nonce-variance-strict".to_string(),
                percent: -5.4,
            }),
            Rc::new(MinimumNonceVarianceAdjuster { minimum: 3.5 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 33750.7,
    }
}
