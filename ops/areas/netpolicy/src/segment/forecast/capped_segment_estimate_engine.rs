use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped segment estimate calculation.
pub trait CappedSegmentEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSegmentEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedSegmentEstimateAdjuster {
    pub step: f64,
}

impl CappedSegmentEstimateStep for RoundingCappedSegmentEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedSegmentEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedSegmentEstimateAdjuster {
    pub percent: f64,
}

impl CappedSegmentEstimateStep for PercentCappedSegmentEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedSegmentEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedSegmentEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedSegmentEstimateStep for TieredCappedSegmentEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedSegmentEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped segment estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSegmentEstimateEngine {
    pub steps: Vec<Rc<dyn CappedSegmentEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSegmentEstimateEngine {
    pub fn evaluate(&mut self, item: &CappedSegmentEstimate) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedSegmentEstimateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedSegmentEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_segment_estimate_engine() -> CappedSegmentEstimateEngine {
    CappedSegmentEstimateEngine {
        steps: vec![
            Rc::new(RoundingCappedSegmentEstimateAdjuster { step: 0.5 }) as Rc<dyn CappedSegmentEstimateStep>,
            Rc::new(PercentCappedSegmentEstimateAdjuster { percent: 8.3 }),
            Rc::new(TieredCappedSegmentEstimateAdjuster {
                bounds: vec![1523.0, 2676.0, 5208.0, 6429.0],
                rates: vec![1.18, 1.13, 1.17, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 19463.2,
    }
}
