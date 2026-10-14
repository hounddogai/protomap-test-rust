use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo ledger calculation.
pub trait SloLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloLedgerAdjuster {
    pub minimum: f64,
}

impl SloLedgerFactor for MinimumSloLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloLedgerAdjuster {
    pub cap: f64,
}

impl SloLedgerFactor for CapSloLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloLedgerAdjuster {
    pub percent: f64,
}

impl SloLedgerFactor for PercentSloLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSloLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloLedgerFactor for SourceSloLedgerAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SloLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloLedgerFactor for TieredSloLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloLedgerFactor for FlaggedSloLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the slo ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloLedgerEngine {
    pub steps: Vec<Rc<dyn SloLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloLedgerEngine {
    pub fn evaluate(&mut self, item: &SloLedger) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SloLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_slo_ledger_engine(flags: &flags::Registry) -> SloLedgerEngine {
    SloLedgerEngine {
        steps: vec![
            Rc::new(MinimumSloLedgerAdjuster { minimum: 9.6 }) as Rc<dyn SloLedgerFactor>,
            Rc::new(CapSloLedgerAdjuster { cap: 46496.0 }),
            Rc::new(PercentSloLedgerAdjuster { percent: 2.5 }),
            Rc::new(SourceSloLedgerAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.81), ("west".to_string(), 1.03)]),
            }),
            Rc::new(TieredSloLedgerAdjuster {
                bounds: vec![4117.0, 8314.0],
                rates: vec![1.17, 1.17],
            }),
            Rc::new(FlaggedSloLedgerAdjuster {
                flags: flags.clone(),
                flag: "slo-ledger-legacy".to_string(),
                percent: 9.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 84527.9,
    }
}
