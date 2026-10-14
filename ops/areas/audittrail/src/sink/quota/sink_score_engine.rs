use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink score calculation.
pub trait SinkScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkScoreMarginOffsetAdjuster {
    pub weight: f64,
}

impl SinkScoreStep for SinkScoreMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &SinkScore, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleSinkScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkScoreStep for LocaleSinkScoreAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &SinkScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkScoreAdjuster {
    pub cap: f64,
}

impl SinkScoreStep for CapSinkScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkScoreAdjuster {
    pub minimum: f64,
}

impl SinkScoreStep for MinimumSinkScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the sink score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkScoreEngine {
    pub steps: Vec<Rc<dyn SinkScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkScoreEngine {
    pub fn run(&mut self, item: &SinkScore) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[SinkScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_sink_score_engine() -> SinkScoreEngine {
    SinkScoreEngine {
        steps: vec![
            Rc::new(SinkScoreMarginOffsetAdjuster { weight: 1.29 }) as Rc<dyn SinkScoreStep>,
            Rc::new(LocaleSinkScoreAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.15)]),
            }),
            Rc::new(CapSinkScoreAdjuster { cap: 24898.0 }),
            Rc::new(MinimumSinkScoreAdjuster { minimum: 19.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 95493.6,
    }
}
