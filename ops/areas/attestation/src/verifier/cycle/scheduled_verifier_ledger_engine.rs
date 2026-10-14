use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled verifier ledger calculation.
pub trait ScheduledVerifierLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledVerifierLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryScheduledVerifierLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledVerifierLedgerAdjuster for CountryScheduledVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ScheduledVerifierLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledVerifierLedgerAdjuster {
    pub percent: f64,
}

impl ScheduledVerifierLedgerAdjuster for PercentScheduledVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledVerifierLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledVerifierLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledVerifierLedgerAdjuster for FlaggedScheduledVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledVerifierLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledVerifierLedgerAdjuster {
    pub cap: f64,
}

impl ScheduledVerifierLedgerAdjuster for CapScheduledVerifierLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledVerifierLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled verifier ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledVerifierLedgerEngine {
    pub steps: Vec<Rc<dyn ScheduledVerifierLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledVerifierLedgerEngine {
    pub fn price(&mut self, item: &ScheduledVerifierLedger) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledVerifierLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_verifier_ledger_engine(flags: &flags::Registry) -> ScheduledVerifierLedgerEngine {
    ScheduledVerifierLedgerEngine {
        steps: vec![
            Rc::new(CountryScheduledVerifierLedgerAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.15), ("central".to_string(), 1.22)]),
            }) as Rc<dyn ScheduledVerifierLedgerAdjuster>,
            Rc::new(PercentScheduledVerifierLedgerAdjuster { percent: -9.2 }),
            Rc::new(FlaggedScheduledVerifierLedgerAdjuster {
                flags: flags.clone(),
                flag: "scheduled-verifier-ledger-strict".to_string(),
                percent: -1.4,
            }),
            Rc::new(CapScheduledVerifierLedgerAdjuster { cap: 45867.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 36127.8,
    }
}
