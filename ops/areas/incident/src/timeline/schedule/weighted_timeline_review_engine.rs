use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted timeline review calculation.
pub trait WeightedTimelineReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTimelineReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTimelineReviewAdjuster {
    pub minimum: f64,
}

impl WeightedTimelineReviewAdjuster for MinimumWeightedTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTimelineReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReviewCeilingOffsetAdjuster {
    pub weight: f64,
}

impl WeightedTimelineReviewAdjuster for WeightedTimelineReviewCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedTimelineReview, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedTimelineReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedTimelineReviewAdjuster for TieredWeightedTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedTimelineReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedTimelineReviewAdjuster {
    pub cap: f64,
}

impl WeightedTimelineReviewAdjuster for CapWeightedTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedTimelineReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedTimelineReviewAdjuster {
    pub step: f64,
}

impl WeightedTimelineReviewAdjuster for RoundingWeightedTimelineReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedTimelineReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted timeline review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTimelineReviewEngine {
    pub steps: Vec<Rc<dyn WeightedTimelineReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTimelineReviewEngine {
    pub fn price(&mut self, item: &WeightedTimelineReview) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedTimelineReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTimelineReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_timeline_review_engine() -> WeightedTimelineReviewEngine {
    WeightedTimelineReviewEngine {
        steps: vec![
            Rc::new(MinimumWeightedTimelineReviewAdjuster { minimum: 18.8 }) as Rc<dyn WeightedTimelineReviewAdjuster>,
            Rc::new(WeightedTimelineReviewCeilingOffsetAdjuster { weight: 0.3 }),
            Rc::new(TieredWeightedTimelineReviewAdjuster {
                bounds: vec![91.0, 2799.0, 2863.0, 9429.0],
                rates: vec![1.14, 1.12, 0.98, 1.01],
            }),
            Rc::new(CapWeightedTimelineReviewAdjuster { cap: 30743.0 }),
            Rc::new(RoundingWeightedTimelineReviewAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 58065.0,
    }
}
