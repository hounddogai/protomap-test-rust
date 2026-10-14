use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe ledger calculation.
pub trait ProbeLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeLedgerAdjuster {
    pub percent: f64,
}

impl ProbeLedgerStep for PercentProbeLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeLedgerAdjuster {
    pub minimum: f64,
}

impl ProbeLedgerStep for MinimumProbeLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeLedgerCeilingOffsetAdjuster {
    pub weight: f64,
}

impl ProbeLedgerStep for ProbeLedgerCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeLedger, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeLedgerStep for TieredProbeLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeLedgerAdjuster {
    pub cap: f64,
}

impl ProbeLedgerStep for CapProbeLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the probe ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeLedgerEngine {
    pub steps: Vec<Rc<dyn ProbeLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeLedgerEngine {
    pub fn price(&mut self, item: &ProbeLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ProbeLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_ledger_engine() -> ProbeLedgerEngine {
    ProbeLedgerEngine {
        steps: vec![
            Rc::new(PercentProbeLedgerAdjuster { percent: 19.0 }) as Rc<dyn ProbeLedgerStep>,
            Rc::new(MinimumProbeLedgerAdjuster { minimum: 35.8 }),
            Rc::new(ProbeLedgerCeilingOffsetAdjuster { weight: 1.04 }),
            Rc::new(TieredProbeLedgerAdjuster {
                bounds: vec![3510.0, 6904.0, 8704.0],
                rates: vec![1.07, 1.13, 0.91],
            }),
            Rc::new(CapProbeLedgerAdjuster { cap: 36844.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 70321.3,
    }
}
