use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook entry calculation.
pub trait RunbookEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookEntryAdjuster {
    pub percent: f64,
}

impl RunbookEntryAdjuster for PercentRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRunbookEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookEntryAdjuster for CountryRunbookEntryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RunbookEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookEntryAdjuster {
    pub step: f64,
}

impl RunbookEntryAdjuster for RoundingRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the runbook entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookEntryEngine {
    pub steps: Vec<Rc<dyn RunbookEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookEntryEngine {
    pub fn evaluate(&mut self, item: &RunbookEntry) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RunbookEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_entry_engine() -> RunbookEntryEngine {
    RunbookEntryEngine {
        steps: vec![
            Rc::new(PercentRunbookEntryAdjuster { percent: 13.3 }) as Rc<dyn RunbookEntryAdjuster>,
            Rc::new(CountryRunbookEntryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.97), ("south".to_string(), 0.83)]),
            }),
            Rc::new(RoundingRunbookEntryAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 52020.1,
    }
}
