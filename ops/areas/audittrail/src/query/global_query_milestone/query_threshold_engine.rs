use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the query threshold calculation.
pub trait QueryThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryThresholdAdjuster {
    pub step: f64,
}

impl QueryThresholdFactor for RoundingQueryThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryThresholdStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl QueryThresholdFactor for QueryThresholdStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &QueryThreshold, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryThresholdAdjuster {
    pub percent: f64,
}

impl QueryThresholdFactor for PercentQueryThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryThresholdEngine {
    pub steps: Vec<Rc<dyn QueryThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryThresholdEngine {
    pub fn run(&mut self, item: &QueryThreshold) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_threshold_engine() -> QueryThresholdEngine {
    QueryThresholdEngine {
        steps: vec![
            Rc::new(RoundingQueryThresholdAdjuster { step: 1.0 }) as Rc<dyn QueryThresholdFactor>,
            Rc::new(QueryThresholdStartsAtOffsetAdjuster { weight: 1.17 }),
            Rc::new(PercentQueryThresholdAdjuster { percent: 22.6 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 82341.1,
    }
}
