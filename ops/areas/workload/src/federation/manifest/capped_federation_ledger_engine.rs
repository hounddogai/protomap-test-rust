use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped federation ledger calculation.
pub trait CappedFederationLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedFederationLedger, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedFederationLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedFederationLedgerModifier for FlaggedCappedFederationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedFederationLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedFederationLedgerAdjuster {
    pub cap: f64,
}

impl CappedFederationLedgerModifier for CapCappedFederationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedFederationLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedFederationLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedFederationLedgerModifier for TieredCappedFederationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedFederationLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped federation ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedFederationLedgerEngine {
    pub steps: Vec<Rc<dyn CappedFederationLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedFederationLedgerEngine {
    pub fn price(&mut self, item: &CappedFederationLedger) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedFederationLedgerModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedFederationLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_federation_ledger_engine(flags: &flags::Registry) -> CappedFederationLedgerEngine {
    CappedFederationLedgerEngine {
        steps: vec![
            Rc::new(FlaggedCappedFederationLedgerAdjuster {
                flags: flags.clone(),
                flag: "capped-federation-ledger-default".to_string(),
                percent: -7.5,
            }) as Rc<dyn CappedFederationLedgerModifier>,
            Rc::new(CapCappedFederationLedgerAdjuster { cap: 34803.0 }),
            Rc::new(TieredCappedFederationLedgerAdjuster {
                bounds: vec![3596.0, 6050.0, 6721.0],
                rates: vec![0.93, 1.07, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 88379.3,
    }
}
