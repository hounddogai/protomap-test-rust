use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress entry calculation.
pub trait EgressEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressEntryAdjuster {
    pub step: f64,
}

impl EgressEntryStep for RoundingEgressEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameEgressEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressEntryStep for NameEgressEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &EgressEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressEntryAdjuster {
    pub cap: f64,
}

impl EgressEntryStep for CapEgressEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEntryBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl EgressEntryStep for EgressEntryBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &EgressEntry, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

/// Runs the egress entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressEntryEngine {
    pub steps: Vec<Rc<dyn EgressEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressEntryEngine {
    pub fn price(&mut self, item: &EgressEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EgressEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_egress_entry_engine() -> EgressEntryEngine {
    EgressEntryEngine {
        steps: vec![
            Rc::new(RoundingEgressEntryAdjuster { step: 0.05 }) as Rc<dyn EgressEntryStep>,
            Rc::new(NameEgressEntryAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.37),
                    ("north".to_string(), 1.39),
                    ("online".to_string(), 0.75),
                ]),
            }),
            Rc::new(CapEgressEntryAdjuster { cap: 30583.0 }),
            Rc::new(EgressEntryBatchSizeOffsetAdjuster { weight: 0.6 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 68060.8,
    }
}
