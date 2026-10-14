use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the sink ledger calculation.
pub trait SinkLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkLedgerAdjuster {
    pub cap: f64,
}

impl SinkLedgerStep for CapSinkLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkLedgerAdjuster {
    pub percent: f64,
}

impl SinkLedgerStep for PercentSinkLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkLedgerAdjuster {
    pub minimum: f64,
}

impl SinkLedgerStep for MinimumSinkLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the sink ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkLedgerEngine {
    pub steps: Vec<Rc<dyn SinkLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkLedgerEngine {
    pub fn evaluate(&mut self, item: &SinkLedger) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_sink_ledger_engine() -> SinkLedgerEngine {
    SinkLedgerEngine {
        steps: vec![
            Rc::new(CapSinkLedgerAdjuster { cap: 42782.0 }) as Rc<dyn SinkLedgerStep>,
            Rc::new(PercentSinkLedgerAdjuster { percent: -23.2 }),
            Rc::new(MinimumSinkLedgerAdjuster { minimum: 26.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 41933.3,
    }
}
