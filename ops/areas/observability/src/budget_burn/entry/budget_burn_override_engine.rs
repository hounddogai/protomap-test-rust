use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the budget burn override calculation.
pub trait BudgetBurnOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnOverrideCapacityOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnOverrideAdjuster for BudgetBurnOverrideCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnOverride, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnOverrideAdjuster {
    pub minimum: f64,
}

impl BudgetBurnOverrideAdjuster for MinimumBudgetBurnOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnOverrideAdjuster for TieredBudgetBurnOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the budget burn override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnOverrideEngine {
    pub steps: Vec<Rc<dyn BudgetBurnOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnOverrideEngine {
    pub fn price(&mut self, item: &BudgetBurnOverride) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_override_engine() -> BudgetBurnOverrideEngine {
    BudgetBurnOverrideEngine {
        steps: vec![
            Rc::new(BudgetBurnOverrideCapacityOffsetAdjuster { weight: 0.26 }) as Rc<dyn BudgetBurnOverrideAdjuster>,
            Rc::new(MinimumBudgetBurnOverrideAdjuster { minimum: 11.7 }),
            Rc::new(TieredBudgetBurnOverrideAdjuster {
                bounds: vec![5897.0, 7362.0, 8468.0],
                rates: vec![1.03, 0.99, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 90124.2,
    }
}
