use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the postmortem variance calculation.
pub trait PostmortemVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemVarianceAmountOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemVarianceModifier for PostmortemVarianceAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemVariance, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemVarianceAdjuster {
    pub cap: f64,
}

impl PostmortemVarianceModifier for CapPostmortemVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPostmortemVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemVarianceModifier for CategoryPostmortemVarianceAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PostmortemVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemVarianceAdjuster {
    pub step: f64,
}

impl PostmortemVarianceModifier for RoundingPostmortemVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the postmortem variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemVarianceEngine {
    pub steps: Vec<Rc<dyn PostmortemVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemVarianceEngine {
    pub fn evaluate(&mut self, item: &PostmortemVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PostmortemVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_postmortem_variance_engine() -> PostmortemVarianceEngine {
    PostmortemVarianceEngine {
        steps: vec![
            Rc::new(PostmortemVarianceAmountOffsetAdjuster { weight: 1.83 }) as Rc<dyn PostmortemVarianceModifier>,
            Rc::new(CapPostmortemVarianceAdjuster { cap: 24445.0 }),
            Rc::new(CategoryPostmortemVarianceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.1)]),
            }),
            Rc::new(RoundingPostmortemVarianceAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 61464.9,
    }
}
