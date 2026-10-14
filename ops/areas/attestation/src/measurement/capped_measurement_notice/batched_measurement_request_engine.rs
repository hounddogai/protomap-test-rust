use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched measurement request calculation.
pub trait BatchedMeasurementRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedMeasurementRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedMeasurementRequestAdjuster {
    pub cap: f64,
}

impl BatchedMeasurementRequestStep for CapBatchedMeasurementRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedMeasurementRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedMeasurementRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedMeasurementRequestStep for NameBatchedMeasurementRequestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedMeasurementRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementRequestGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl BatchedMeasurementRequestStep for BatchedMeasurementRequestGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedMeasurementRequest, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the batched measurement request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedMeasurementRequestEngine {
    pub steps: Vec<Rc<dyn BatchedMeasurementRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedMeasurementRequestEngine {
    pub fn run(&mut self, item: &BatchedMeasurementRequest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedMeasurementRequestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedMeasurementRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_measurement_request_engine() -> BatchedMeasurementRequestEngine {
    BatchedMeasurementRequestEngine {
        steps: vec![
            Rc::new(CapBatchedMeasurementRequestAdjuster { cap: 10789.0 }) as Rc<dyn BatchedMeasurementRequestStep>,
            Rc::new(NameBatchedMeasurementRequestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.32), ("retail".to_string(), 1.33)]),
            }),
            Rc::new(BatchedMeasurementRequestGraceDaysOffsetAdjuster { weight: 0.51 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 76914.4,
    }
}
