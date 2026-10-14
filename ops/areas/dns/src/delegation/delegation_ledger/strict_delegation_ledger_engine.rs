use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict delegation ledger calculation.
pub trait StrictDelegationLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictDelegationLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationLedgerThresholdOffsetAdjuster {
    pub weight: f64,
}

impl StrictDelegationLedgerFactor for StrictDelegationLedgerThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &StrictDelegationLedger, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictDelegationLedgerAdjuster {
    pub step: f64,
}

impl StrictDelegationLedgerFactor for RoundingStrictDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictDelegationLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictDelegationLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictDelegationLedgerFactor for FlaggedStrictDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictDelegationLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictDelegationLedgerAdjuster {
    pub percent: f64,
}

impl StrictDelegationLedgerFactor for PercentStrictDelegationLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictDelegationLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict delegation ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictDelegationLedgerEngine {
    pub steps: Vec<Rc<dyn StrictDelegationLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictDelegationLedgerEngine {
    pub fn price(&mut self, item: &StrictDelegationLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictDelegationLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_delegation_ledger_engine(flags: &flags::Registry) -> StrictDelegationLedgerEngine {
    StrictDelegationLedgerEngine {
        steps: vec![
            Rc::new(StrictDelegationLedgerThresholdOffsetAdjuster { weight: 1.5 })
                as Rc<dyn StrictDelegationLedgerFactor>,
            Rc::new(RoundingStrictDelegationLedgerAdjuster { step: 0.5 }),
            Rc::new(FlaggedStrictDelegationLedgerAdjuster {
                flags: flags.clone(),
                flag: "strict-delegation-ledger-tiered".to_string(),
                percent: 4.3,
            }),
            Rc::new(PercentStrictDelegationLedgerAdjuster { percent: -10.1 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 59313.0,
    }
}
