use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage draft calculation.
pub trait UsageDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDraftRatioOffsetAdjuster {
    pub weight: f64,
}

impl UsageDraftFactor for UsageDraftRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &UsageDraft, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageDraftFactor for TieredUsageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageDraftAdjuster {
    pub step: f64,
}

impl UsageDraftFactor for RoundingUsageDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the usage draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageDraftEngine {
    pub steps: Vec<Rc<dyn UsageDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageDraftEngine {
    pub fn run(&mut self, item: &UsageDraft) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[UsageDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_draft_engine() -> UsageDraftEngine {
    UsageDraftEngine {
        steps: vec![
            Rc::new(UsageDraftRatioOffsetAdjuster { weight: 0.61 }) as Rc<dyn UsageDraftFactor>,
            Rc::new(TieredUsageDraftAdjuster {
                bounds: vec![2996.0, 5130.0, 8267.0, 9230.0],
                rates: vec![0.91, 0.84, 1.18, 0.8],
            }),
            Rc::new(RoundingUsageDraftAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 60870.3,
    }
}
