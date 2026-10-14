use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback failover budget calculation.
pub trait FallbackFailoverBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackFailoverBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackFailoverBudgetAdjuster {
    pub cap: f64,
}

impl FallbackFailoverBudgetFactor for CapFallbackFailoverBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackFailoverBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackFailoverBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackFailoverBudgetFactor for TieredFallbackFailoverBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackFailoverBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackFailoverBudgetAdjuster {
    pub minimum: f64,
}

impl FallbackFailoverBudgetFactor for MinimumFallbackFailoverBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackFailoverBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback failover budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackFailoverBudgetEngine {
    pub steps: Vec<Rc<dyn FallbackFailoverBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackFailoverBudgetEngine {
    pub fn run(&mut self, item: &FallbackFailoverBudget) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackFailoverBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_failover_budget_engine() -> FallbackFailoverBudgetEngine {
    FallbackFailoverBudgetEngine {
        steps: vec![
            Rc::new(CapFallbackFailoverBudgetAdjuster { cap: 15762.0 }) as Rc<dyn FallbackFailoverBudgetFactor>,
            Rc::new(TieredFallbackFailoverBudgetAdjuster {
                bounds: vec![6849.0, 7149.0],
                rates: vec![0.99, 0.82],
            }),
            Rc::new(MinimumFallbackFailoverBudgetAdjuster { minimum: 5.2 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 44708.5,
    }
}
