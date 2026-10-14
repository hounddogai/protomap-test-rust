use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the budget burn ledger calculation.
pub trait BudgetBurnLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnLedgerAdjuster {
    pub minimum: f64,
}

impl BudgetBurnLedgerFactor for MinimumBudgetBurnLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnLedgerFactor for TieredBudgetBurnLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnLedgerTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnLedgerFactor for BudgetBurnLedgerTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnLedger, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the budget burn ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnLedgerEngine {
    pub steps: Vec<Rc<dyn BudgetBurnLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnLedgerEngine {
    pub fn evaluate(&mut self, item: &BudgetBurnLedger) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_budget_burn_ledger_engine() -> BudgetBurnLedgerEngine {
    BudgetBurnLedgerEngine {
        steps: vec![
            Rc::new(MinimumBudgetBurnLedgerAdjuster { minimum: 4.0 }) as Rc<dyn BudgetBurnLedgerFactor>,
            Rc::new(TieredBudgetBurnLedgerAdjuster {
                bounds: vec![1293.0, 3491.0, 9358.0, 9606.0],
                rates: vec![0.91, 0.95, 0.95, 1.0],
            }),
            Rc::new(BudgetBurnLedgerTtlSecondsOffsetAdjuster { weight: 0.17 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 43036.8,
    }
}
