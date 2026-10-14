use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal ttl policy ledger calculation.
pub trait SeasonalTtlPolicyLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalTtlPolicyLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalTtlPolicyLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalTtlPolicyLedgerFactor for TieredSeasonalTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalTtlPolicyLedgerAdjuster {
    pub percent: f64,
}

impl SeasonalTtlPolicyLedgerFactor for PercentSeasonalTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalTtlPolicyLedgerAdjuster {
    pub minimum: f64,
}

impl SeasonalTtlPolicyLedgerFactor for MinimumSeasonalTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalTtlPolicyLedgerAdjuster {
    pub step: f64,
}

impl SeasonalTtlPolicyLedgerFactor for RoundingSeasonalTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyLedgerDiscountOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalTtlPolicyLedgerFactor for SeasonalTtlPolicyLedgerDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyLedger, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalTtlPolicyLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalTtlPolicyLedgerFactor for FlaggedSeasonalTtlPolicyLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalTtlPolicyLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal ttl policy ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalTtlPolicyLedgerEngine {
    pub steps: Vec<Rc<dyn SeasonalTtlPolicyLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalTtlPolicyLedgerEngine {
    pub fn evaluate(&mut self, item: &SeasonalTtlPolicyLedger) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalTtlPolicyLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalTtlPolicyLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_ttl_policy_ledger_engine(flags: &flags::Registry) -> SeasonalTtlPolicyLedgerEngine {
    SeasonalTtlPolicyLedgerEngine {
        steps: vec![
            Rc::new(TieredSeasonalTtlPolicyLedgerAdjuster {
                bounds: vec![6974.0, 6995.0, 8400.0, 9185.0],
                rates: vec![1.12, 0.84, 0.83, 1.15],
            }) as Rc<dyn SeasonalTtlPolicyLedgerFactor>,
            Rc::new(PercentSeasonalTtlPolicyLedgerAdjuster { percent: -12.0 }),
            Rc::new(MinimumSeasonalTtlPolicyLedgerAdjuster { minimum: 1.1 }),
            Rc::new(RoundingSeasonalTtlPolicyLedgerAdjuster { step: 0.25 }),
            Rc::new(SeasonalTtlPolicyLedgerDiscountOffsetAdjuster { weight: 0.72 }),
            Rc::new(FlaggedSeasonalTtlPolicyLedgerAdjuster {
                flags: flags.clone(),
                flag: "seasonal-ttl-policy-ledger-seasonal".to_string(),
                percent: 8.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 95126.0,
    }
}
