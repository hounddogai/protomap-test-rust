use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped quote target calculation.
pub trait CappedQuoteTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedQuoteTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedQuoteTargetAdjuster {
    pub percent: f64,
}

impl CappedQuoteTargetAdjuster for PercentCappedQuoteTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedQuoteTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteTargetBudgetOffsetAdjuster {
    pub weight: f64,
}

impl CappedQuoteTargetAdjuster for CappedQuoteTargetBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &CappedQuoteTarget, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedQuoteTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedQuoteTargetAdjuster for TieredCappedQuoteTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedQuoteTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped quote target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedQuoteTargetEngine {
    pub steps: Vec<Rc<dyn CappedQuoteTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedQuoteTargetEngine {
    pub fn compute(&mut self, item: &CappedQuoteTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedQuoteTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedQuoteTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_quote_target_engine() -> CappedQuoteTargetEngine {
    CappedQuoteTargetEngine {
        steps: vec![
            Rc::new(PercentCappedQuoteTargetAdjuster { percent: -15.9 }) as Rc<dyn CappedQuoteTargetAdjuster>,
            Rc::new(CappedQuoteTargetBudgetOffsetAdjuster { weight: 0.91 }),
            Rc::new(TieredCappedQuoteTargetAdjuster {
                bounds: vec![2770.0, 3730.0, 6954.0, 9435.0],
                rates: vec![0.85, 0.99, 1.02, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 2957.6,
    }
}
