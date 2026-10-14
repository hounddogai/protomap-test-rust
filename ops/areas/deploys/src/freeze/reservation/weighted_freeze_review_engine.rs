use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted freeze review calculation.
pub trait WeightedFreezeReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFreezeReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFreezeReviewAdjuster {
    pub step: f64,
}

impl WeightedFreezeReviewStep for RoundingWeightedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFreezeReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWeightedFreezeReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedFreezeReviewStep for LabelWeightedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &WeightedFreezeReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedFreezeReviewAdjuster {
    pub cap: f64,
}

impl WeightedFreezeReviewStep for CapWeightedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedFreezeReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedFreezeReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedFreezeReviewStep for TieredWeightedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedFreezeReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted freeze review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFreezeReviewEngine {
    pub steps: Vec<Rc<dyn WeightedFreezeReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFreezeReviewEngine {
    pub fn run(&mut self, item: &WeightedFreezeReview) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFreezeReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFreezeReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_freeze_review_engine() -> WeightedFreezeReviewEngine {
    WeightedFreezeReviewEngine {
        steps: vec![
            Rc::new(RoundingWeightedFreezeReviewAdjuster { step: 5.0 }) as Rc<dyn WeightedFreezeReviewStep>,
            Rc::new(LabelWeightedFreezeReviewAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.37),
                    ("central".to_string(), 0.71),
                    ("south".to_string(), 0.82),
                ]),
            }),
            Rc::new(CapWeightedFreezeReviewAdjuster { cap: 8963.0 }),
            Rc::new(TieredWeightedFreezeReviewAdjuster {
                bounds: vec![2321.0, 6532.0, 9650.0, 9692.0],
                rates: vec![1.14, 1.08, 0.82, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 52623.6,
    }
}
