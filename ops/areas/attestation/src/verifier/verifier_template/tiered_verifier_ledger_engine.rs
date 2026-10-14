use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered verifier ledger calculation.
pub trait TieredVerifierLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredVerifierLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredVerifierLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredVerifierLedgerStep for TieredTieredVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredVerifierLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierLedgerCapacityOffsetAdjuster {
    pub weight: f64,
}

impl TieredVerifierLedgerStep for TieredVerifierLedgerCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &TieredVerifierLedger, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredVerifierLedgerAdjuster {
    pub cap: f64,
}

impl TieredVerifierLedgerStep for CapTieredVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredVerifierLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredVerifierLedgerAdjuster {
    pub step: f64,
}

impl TieredVerifierLedgerStep for RoundingTieredVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredVerifierLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredVerifierLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredVerifierLedgerStep for FlaggedTieredVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredVerifierLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered verifier ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredVerifierLedgerEngine {
    pub steps: Vec<Rc<dyn TieredVerifierLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredVerifierLedgerEngine {
    pub fn compute(&mut self, item: &TieredVerifierLedger) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredVerifierLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredVerifierLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_verifier_ledger_engine(flags: &flags::Registry) -> TieredVerifierLedgerEngine {
    TieredVerifierLedgerEngine {
        steps: vec![
            Rc::new(TieredTieredVerifierLedgerAdjuster {
                bounds: vec![946.0, 1487.0, 2189.0],
                rates: vec![1.08, 0.88, 1.04],
            }) as Rc<dyn TieredVerifierLedgerStep>,
            Rc::new(TieredVerifierLedgerCapacityOffsetAdjuster { weight: 0.59 }),
            Rc::new(CapTieredVerifierLedgerAdjuster { cap: 25297.0 }),
            Rc::new(RoundingTieredVerifierLedgerAdjuster { step: 0.1 }),
            Rc::new(FlaggedTieredVerifierLedgerAdjuster {
                flags: flags.clone(),
                flag: "tiered-verifier-ledger-weighted".to_string(),
                percent: 9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 40942.0,
    }
}
