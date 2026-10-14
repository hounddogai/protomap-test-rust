use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the postmortem baseline calculation.
pub trait PostmortemBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBaselineThresholdOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemBaselineStep for PostmortemBaselineThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemBaseline, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemBaselineAdjuster {
    pub percent: f64,
}

impl PostmortemBaselineStep for PercentPostmortemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemBaselineAdjuster {
    pub cap: f64,
}

impl PostmortemBaselineStep for CapPostmortemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the postmortem baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemBaselineEngine {
    pub steps: Vec<Rc<dyn PostmortemBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemBaselineEngine {
    pub fn price(&mut self, item: &PostmortemBaseline) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_baseline_engine() -> PostmortemBaselineEngine {
    PostmortemBaselineEngine {
        steps: vec![
            Rc::new(PostmortemBaselineThresholdOffsetAdjuster { weight: 1.97 }) as Rc<dyn PostmortemBaselineStep>,
            Rc::new(PercentPostmortemBaselineAdjuster { percent: 16.1 }),
            Rc::new(CapPostmortemBaselineAdjuster { cap: 49790.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 95318.5,
    }
}
