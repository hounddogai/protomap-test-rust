use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched freeze segment calculation.
pub trait BatchedFreezeSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedFreezeSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBatchedFreezeSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedFreezeSegmentAdjuster for CountryBatchedFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BatchedFreezeSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedFreezeSegmentAdjuster {
    pub minimum: f64,
}

impl BatchedFreezeSegmentAdjuster for MinimumBatchedFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedFreezeSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedFreezeSegmentAdjuster {
    pub step: f64,
}

impl BatchedFreezeSegmentAdjuster for RoundingBatchedFreezeSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedFreezeSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched freeze segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedFreezeSegmentEngine {
    pub steps: Vec<Rc<dyn BatchedFreezeSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedFreezeSegmentEngine {
    pub fn run(&mut self, item: &BatchedFreezeSegment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedFreezeSegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedFreezeSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_freeze_segment_engine() -> BatchedFreezeSegmentEngine {
    BatchedFreezeSegmentEngine {
        steps: vec![
            Rc::new(CountryBatchedFreezeSegmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.93)]),
            }) as Rc<dyn BatchedFreezeSegmentAdjuster>,
            Rc::new(MinimumBatchedFreezeSegmentAdjuster { minimum: 6.1 }),
            Rc::new(RoundingBatchedFreezeSegmentAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 27744.8,
    }
}
