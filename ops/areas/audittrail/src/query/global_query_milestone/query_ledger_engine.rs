use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the query ledger calculation.
pub trait QueryLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryLedgerAdjuster {
    pub cap: f64,
}

impl QueryLedgerModifier for CapQueryLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryLedgerAdjuster {
    pub step: f64,
}

impl QueryLedgerModifier for RoundingQueryLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitQueryLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryLedgerModifier for UnitQueryLedgerAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &QueryLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the query ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryLedgerEngine {
    pub steps: Vec<Rc<dyn QueryLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryLedgerEngine {
    pub fn compute(&mut self, item: &QueryLedger) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[QueryLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_ledger_engine() -> QueryLedgerEngine {
    QueryLedgerEngine {
        steps: vec![
            Rc::new(CapQueryLedgerAdjuster { cap: 5805.0 }) as Rc<dyn QueryLedgerModifier>,
            Rc::new(RoundingQueryLedgerAdjuster { step: 5.0 }),
            Rc::new(UnitQueryLedgerAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.97)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 22392.8,
    }
}
