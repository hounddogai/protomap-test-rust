use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary failover ledger calculation.
pub trait PrimaryFailoverLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFailoverLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryFailoverLedgerAdjuster {
    pub step: f64,
}

impl PrimaryFailoverLedgerAdjuster for RoundingPrimaryFailoverLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryFailoverLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryFailoverLedgerAdjuster {
    pub minimum: f64,
}

impl PrimaryFailoverLedgerAdjuster for MinimumPrimaryFailoverLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryFailoverLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverLedgerAmountOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryFailoverLedgerAdjuster for PrimaryFailoverLedgerAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryFailoverLedger, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryFailoverLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryFailoverLedgerAdjuster for FlaggedPrimaryFailoverLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryFailoverLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPrimaryFailoverLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryFailoverLedgerAdjuster for UnitPrimaryFailoverLedgerAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PrimaryFailoverLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryFailoverLedgerAdjuster {
    pub cap: f64,
}

impl PrimaryFailoverLedgerAdjuster for CapPrimaryFailoverLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryFailoverLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary failover ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverLedgerEngine {
    pub steps: Vec<Rc<dyn PrimaryFailoverLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFailoverLedgerEngine {
    pub fn compute(&mut self, item: &PrimaryFailoverLedger) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFailoverLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_failover_ledger_engine(flags: &flags::Registry) -> PrimaryFailoverLedgerEngine {
    PrimaryFailoverLedgerEngine {
        steps: vec![
            Rc::new(RoundingPrimaryFailoverLedgerAdjuster { step: 1.0 }) as Rc<dyn PrimaryFailoverLedgerAdjuster>,
            Rc::new(MinimumPrimaryFailoverLedgerAdjuster { minimum: 23.9 }),
            Rc::new(PrimaryFailoverLedgerAmountOffsetAdjuster { weight: 1.51 }),
            Rc::new(FlaggedPrimaryFailoverLedgerAdjuster {
                flags: flags.clone(),
                flag: "primary-failover-ledger-strict".to_string(),
                percent: -3.1,
            }),
            Rc::new(UnitPrimaryFailoverLedgerAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.92),
                    ("east".to_string(), 0.81),
                    ("retail".to_string(), 0.71),
                ]),
            }),
            Rc::new(CapPrimaryFailoverLedgerAdjuster { cap: 18077.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 31376.1,
    }
}
