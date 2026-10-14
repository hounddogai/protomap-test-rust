use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline ledger calculation.
pub trait TimelineLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineLedgerAdjuster {
    pub cap: f64,
}

impl TimelineLedgerModifier for CapTimelineLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineLedgerAdjuster {
    pub percent: f64,
}

impl TimelineLedgerModifier for PercentTimelineLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TimelineLedgerModifier for TieredTimelineLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TimelineLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLedgerWeightOffsetAdjuster {
    pub weight: f64,
}

impl TimelineLedgerModifier for TimelineLedgerWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineLedger, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTimelineLedgerAdjuster {
    pub step: f64,
}

impl TimelineLedgerModifier for RoundingTimelineLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TimelineLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the timeline ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineLedgerEngine {
    pub steps: Vec<Rc<dyn TimelineLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineLedgerEngine {
    pub fn evaluate(&mut self, item: &TimelineLedger) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[TimelineLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_timeline_ledger_engine() -> TimelineLedgerEngine {
    TimelineLedgerEngine {
        steps: vec![
            Rc::new(CapTimelineLedgerAdjuster { cap: 40259.0 }) as Rc<dyn TimelineLedgerModifier>,
            Rc::new(PercentTimelineLedgerAdjuster { percent: -14.0 }),
            Rc::new(TieredTimelineLedgerAdjuster {
                bounds: vec![397.0, 6112.0],
                rates: vec![1.2, 0.81],
            }),
            Rc::new(TimelineLedgerWeightOffsetAdjuster { weight: 0.12 }),
            Rc::new(RoundingTimelineLedgerAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 5358.8,
    }
}
