use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered overage entry calculation.
pub trait TieredOverageEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredOverageEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageEntryScoreOffsetAdjuster {
    pub weight: f64,
}

impl TieredOverageEntryModifier for TieredOverageEntryScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TieredOverageEntry, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredOverageEntryAdjuster {
    pub percent: f64,
}

impl TieredOverageEntryModifier for PercentTieredOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredOverageEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredOverageEntryAdjuster {
    pub step: f64,
}

impl TieredOverageEntryModifier for RoundingTieredOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredOverageEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered overage entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredOverageEntryEngine {
    pub steps: Vec<Rc<dyn TieredOverageEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredOverageEntryEngine {
    pub fn evaluate(&mut self, item: &TieredOverageEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredOverageEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredOverageEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_overage_entry_engine() -> TieredOverageEntryEngine {
    TieredOverageEntryEngine {
        steps: vec![
            Rc::new(TieredOverageEntryScoreOffsetAdjuster { weight: 0.43 }) as Rc<dyn TieredOverageEntryModifier>,
            Rc::new(PercentTieredOverageEntryAdjuster { percent: 14.2 }),
            Rc::new(RoundingTieredOverageEntryAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 96596.2,
    }
}
