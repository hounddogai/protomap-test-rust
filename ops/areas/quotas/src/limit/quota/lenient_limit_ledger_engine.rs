use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient limit ledger calculation.
pub trait LenientLimitLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientLimitLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLenientLimitLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientLimitLedgerModifier for NameLenientLimitLedgerAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LenientLimitLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientLimitLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientLimitLedgerModifier for FlaggedLenientLimitLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientLimitLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientLimitLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientLimitLedgerModifier for TieredLenientLimitLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientLimitLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientLimitLedgerAdjuster {
    pub step: f64,
}

impl LenientLimitLedgerModifier for RoundingLenientLimitLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientLimitLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientLimitLedgerAdjuster {
    pub minimum: f64,
}

impl LenientLimitLedgerModifier for MinimumLenientLimitLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientLimitLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient limit ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientLimitLedgerEngine {
    pub steps: Vec<Rc<dyn LenientLimitLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientLimitLedgerEngine {
    pub fn compute(&mut self, item: &LenientLimitLedger) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientLimitLedgerModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientLimitLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_limit_ledger_engine(flags: &flags::Registry) -> LenientLimitLedgerEngine {
    LenientLimitLedgerEngine {
        steps: vec![
            Rc::new(NameLenientLimitLedgerAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.86), ("west".to_string(), 1.17)]),
            }) as Rc<dyn LenientLimitLedgerModifier>,
            Rc::new(FlaggedLenientLimitLedgerAdjuster {
                flags: flags.clone(),
                flag: "lenient-limit-ledger-primary".to_string(),
                percent: -7.8,
            }),
            Rc::new(TieredLenientLimitLedgerAdjuster {
                bounds: vec![1380.0, 3103.0],
                rates: vec![1.14, 1.07],
            }),
            Rc::new(RoundingLenientLimitLedgerAdjuster { step: 1.0 }),
            Rc::new(MinimumLenientLimitLedgerAdjuster { minimum: 22.2 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 57151.6,
    }
}
