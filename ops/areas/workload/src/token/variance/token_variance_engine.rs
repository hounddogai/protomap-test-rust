use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token variance calculation.
pub trait TokenVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenVarianceStep for TieredTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenVarianceAdjuster {
    pub cap: f64,
}

impl TokenVarianceStep for CapTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenVarianceAmountOffsetAdjuster {
    pub weight: f64,
}

impl TokenVarianceStep for TokenVarianceAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TokenVariance, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenVarianceAdjuster {
    pub step: f64,
}

impl TokenVarianceStep for RoundingTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the token variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenVarianceEngine {
    pub steps: Vec<Rc<dyn TokenVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenVarianceEngine {
    pub fn run(&mut self, item: &TokenVariance) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TokenVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_token_variance_engine() -> TokenVarianceEngine {
    TokenVarianceEngine {
        steps: vec![
            Rc::new(TieredTokenVarianceAdjuster {
                bounds: vec![201.0, 9338.0],
                rates: vec![0.85, 1.11],
            }) as Rc<dyn TokenVarianceStep>,
            Rc::new(CapTokenVarianceAdjuster { cap: 28877.0 }),
            Rc::new(TokenVarianceAmountOffsetAdjuster { weight: 1.31 }),
            Rc::new(RoundingTokenVarianceAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 89320.8,
    }
}
