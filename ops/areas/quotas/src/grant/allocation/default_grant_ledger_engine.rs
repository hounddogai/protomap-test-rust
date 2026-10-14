use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default grant ledger calculation.
pub trait DefaultGrantLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultGrantLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultGrantLedgerAdjuster {
    pub cap: f64,
}

impl DefaultGrantLedgerFactor for CapDefaultGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultGrantLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultGrantLedgerAdjuster {
    pub percent: f64,
}

impl DefaultGrantLedgerFactor for PercentDefaultGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultGrantLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultGrantLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultGrantLedgerFactor for TieredDefaultGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultGrantLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantLedgerRatioOffsetAdjuster {
    pub weight: f64,
}

impl DefaultGrantLedgerFactor for DefaultGrantLedgerRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultGrantLedger, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultGrantLedgerAdjuster {
    pub minimum: f64,
}

impl DefaultGrantLedgerFactor for MinimumDefaultGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultGrantLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the default grant ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultGrantLedgerEngine {
    pub steps: Vec<Rc<dyn DefaultGrantLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultGrantLedgerEngine {
    pub fn evaluate(&mut self, item: &DefaultGrantLedger) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultGrantLedgerFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultGrantLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_grant_ledger_engine() -> DefaultGrantLedgerEngine {
    DefaultGrantLedgerEngine {
        steps: vec![
            Rc::new(CapDefaultGrantLedgerAdjuster { cap: 28005.0 }) as Rc<dyn DefaultGrantLedgerFactor>,
            Rc::new(PercentDefaultGrantLedgerAdjuster { percent: 18.7 }),
            Rc::new(TieredDefaultGrantLedgerAdjuster {
                bounds: vec![2614.0, 3010.0, 5349.0, 6361.0],
                rates: vec![0.88, 1.09, 0.98, 1.15],
            }),
            Rc::new(DefaultGrantLedgerRatioOffsetAdjuster { weight: 0.12 }),
            Rc::new(MinimumDefaultGrantLedgerAdjuster { minimum: 33.5 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 19748.7,
    }
}
