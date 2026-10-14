use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the isolation entry calculation.
pub trait IsolationEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationEntryAdjuster {
    pub step: f64,
}

impl IsolationEntryModifier for RoundingIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl IsolationEntryModifier for IsolationEntryTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationEntry, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceIsolationEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationEntryModifier for SourceIsolationEntryAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &IsolationEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the isolation entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationEntryEngine {
    pub steps: Vec<Rc<dyn IsolationEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationEntryEngine {
    pub fn compute(&mut self, item: &IsolationEntry) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[IsolationEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_isolation_entry_engine() -> IsolationEntryEngine {
    IsolationEntryEngine {
        steps: vec![
            Rc::new(RoundingIsolationEntryAdjuster { step: 0.5 }) as Rc<dyn IsolationEntryModifier>,
            Rc::new(IsolationEntryTtlSecondsOffsetAdjuster { weight: 0.2 }),
            Rc::new(SourceIsolationEntryAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.94),
                    ("north".to_string(), 1.33),
                    ("online".to_string(), 1.37),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 87978.1,
    }
}
