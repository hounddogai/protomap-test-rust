use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual sink request calculation.
pub trait ManualSinkRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSinkRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualSinkRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualSinkRequestStep for TieredManualSinkRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualSinkRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSinkRequestAdjuster {
    pub cap: f64,
}

impl ManualSinkRequestStep for CapManualSinkRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSinkRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSinkRequestFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl ManualSinkRequestStep for ManualSinkRequestFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &ManualSinkRequest, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualSinkRequestAdjuster {
    pub step: f64,
}

impl ManualSinkRequestStep for RoundingManualSinkRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualSinkRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual sink request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSinkRequestEngine {
    pub steps: Vec<Rc<dyn ManualSinkRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSinkRequestEngine {
    pub fn evaluate(&mut self, item: &ManualSinkRequest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualSinkRequestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualSinkRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_sink_request_engine() -> ManualSinkRequestEngine {
    ManualSinkRequestEngine {
        steps: vec![
            Rc::new(TieredManualSinkRequestAdjuster {
                bounds: vec![1487.0, 2303.0],
                rates: vec![0.86, 1.09],
            }) as Rc<dyn ManualSinkRequestStep>,
            Rc::new(CapManualSinkRequestAdjuster { cap: 12138.0 }),
            Rc::new(ManualSinkRequestFloorValueOffsetAdjuster { weight: 0.53 }),
            Rc::new(RoundingManualSinkRequestAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 26653.1,
    }
}
