use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched health check segment calculation.
pub trait BatchedHealthCheckSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedHealthCheckSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedHealthCheckSegmentAdjuster {
    pub minimum: f64,
}

impl BatchedHealthCheckSegmentAdjuster for MinimumBatchedHealthCheckSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedHealthCheckSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedHealthCheckSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedHealthCheckSegmentAdjuster for TieredBatchedHealthCheckSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedHealthCheckSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedHealthCheckSegmentAdjuster {
    pub step: f64,
}

impl BatchedHealthCheckSegmentAdjuster for RoundingBatchedHealthCheckSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedHealthCheckSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched health check segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedHealthCheckSegmentEngine {
    pub steps: Vec<Rc<dyn BatchedHealthCheckSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedHealthCheckSegmentEngine {
    pub fn evaluate(&mut self, item: &BatchedHealthCheckSegment) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedHealthCheckSegmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedHealthCheckSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_health_check_segment_engine() -> BatchedHealthCheckSegmentEngine {
    BatchedHealthCheckSegmentEngine {
        steps: vec![
            Rc::new(MinimumBatchedHealthCheckSegmentAdjuster { minimum: 5.3 })
                as Rc<dyn BatchedHealthCheckSegmentAdjuster>,
            Rc::new(TieredBatchedHealthCheckSegmentAdjuster {
                bounds: vec![238.0, 6850.0],
                rates: vec![1.14, 1.09],
            }),
            Rc::new(RoundingBatchedHealthCheckSegmentAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 91314.6,
    }
}
