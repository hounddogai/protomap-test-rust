use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered grant ledger calculation.
pub trait TieredGrantLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredGrantLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantLedgerPriorityOffsetAdjuster {
    pub weight: f64,
}

impl TieredGrantLedgerFactor for TieredGrantLedgerPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &TieredGrantLedger, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredGrantLedgerAdjuster {
    pub step: f64,
}

impl TieredGrantLedgerFactor for RoundingTieredGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredGrantLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredGrantLedgerAdjuster {
    pub minimum: f64,
}

impl TieredGrantLedgerFactor for MinimumTieredGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredGrantLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredGrantLedgerAdjuster {
    pub cap: f64,
}

impl TieredGrantLedgerFactor for CapTieredGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredGrantLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredGrantLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredGrantLedgerFactor for TieredTieredGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredGrantLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered grant ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredGrantLedgerEngine {
    pub steps: Vec<Rc<dyn TieredGrantLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredGrantLedgerEngine {
    pub fn compute(&mut self, item: &TieredGrantLedger) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredGrantLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredGrantLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_grant_ledger_engine() -> TieredGrantLedgerEngine {
    TieredGrantLedgerEngine {
        steps: vec![
            Rc::new(TieredGrantLedgerPriorityOffsetAdjuster { weight: 1.66 }) as Rc<dyn TieredGrantLedgerFactor>,
            Rc::new(RoundingTieredGrantLedgerAdjuster { step: 0.25 }),
            Rc::new(MinimumTieredGrantLedgerAdjuster { minimum: 3.4 }),
            Rc::new(CapTieredGrantLedgerAdjuster { cap: 27099.0 }),
            Rc::new(TieredTieredGrantLedgerAdjuster {
                bounds: vec![136.0, 2195.0, 8355.0],
                rates: vec![0.97, 0.89, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 47860.1,
    }
}
