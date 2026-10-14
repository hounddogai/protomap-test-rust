use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token adjustment calculation.
pub trait TokenAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenAdjustmentAdjuster {
    pub percent: f64,
}

impl TokenAdjustmentModifier for PercentTokenAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenAdjustmentAdjuster {
    pub minimum: f64,
}

impl TokenAdjustmentModifier for MinimumTokenAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenAdjustmentAdjuster {
    pub step: f64,
}

impl TokenAdjustmentModifier for RoundingTokenAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenAdjustmentModifier for TieredTokenAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenAdjustmentAdjuster {
    pub cap: f64,
}

impl TokenAdjustmentModifier for CapTokenAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTokenAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenAdjustmentModifier for NameTokenAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TokenAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the token adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenAdjustmentEngine {
    pub steps: Vec<Rc<dyn TokenAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenAdjustmentEngine {
    pub fn price(&mut self, item: &TokenAdjustment) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TokenAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TokenAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_token_adjustment_engine() -> TokenAdjustmentEngine {
    TokenAdjustmentEngine {
        steps: vec![
            Rc::new(PercentTokenAdjustmentAdjuster { percent: 11.2 }) as Rc<dyn TokenAdjustmentModifier>,
            Rc::new(MinimumTokenAdjustmentAdjuster { minimum: 7.8 }),
            Rc::new(RoundingTokenAdjustmentAdjuster { step: 0.5 }),
            Rc::new(TieredTokenAdjustmentAdjuster {
                bounds: vec![2468.0, 6334.0],
                rates: vec![1.16, 0.98],
            }),
            Rc::new(CapTokenAdjustmentAdjuster { cap: 19767.0 }),
            Rc::new(NameTokenAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.06),
                    ("east".to_string(), 0.7),
                    ("west".to_string(), 0.95),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 32944.2,
    }
}
