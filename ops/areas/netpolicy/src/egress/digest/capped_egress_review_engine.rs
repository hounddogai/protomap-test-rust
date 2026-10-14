use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped egress review calculation.
pub trait CappedEgressReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedEgressReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCappedEgressReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedEgressReviewFactor for SegmentCappedEgressReviewAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &CappedEgressReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedEgressReviewAdjuster {
    pub minimum: f64,
}

impl CappedEgressReviewFactor for MinimumCappedEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedEgressReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedEgressReviewAdjuster {
    pub cap: f64,
}

impl CappedEgressReviewFactor for CapCappedEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedEgressReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedEgressReviewAdjuster {
    pub percent: f64,
}

impl CappedEgressReviewFactor for PercentCappedEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedEgressReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressReviewScoreOffsetAdjuster {
    pub weight: f64,
}

impl CappedEgressReviewFactor for CappedEgressReviewScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &CappedEgressReview, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedEgressReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedEgressReviewFactor for TieredCappedEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedEgressReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped egress review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedEgressReviewEngine {
    pub steps: Vec<Rc<dyn CappedEgressReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedEgressReviewEngine {
    pub fn evaluate(&mut self, item: &CappedEgressReview) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedEgressReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedEgressReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_egress_review_engine() -> CappedEgressReviewEngine {
    CappedEgressReviewEngine {
        steps: vec![
            Rc::new(SegmentCappedEgressReviewAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.75),
                    ("online".to_string(), 0.85),
                    ("north".to_string(), 0.73),
                ]),
            }) as Rc<dyn CappedEgressReviewFactor>,
            Rc::new(MinimumCappedEgressReviewAdjuster { minimum: 19.3 }),
            Rc::new(CapCappedEgressReviewAdjuster { cap: 9541.0 }),
            Rc::new(PercentCappedEgressReviewAdjuster { percent: 21.0 }),
            Rc::new(CappedEgressReviewScoreOffsetAdjuster { weight: 1.55 }),
            Rc::new(TieredCappedEgressReviewAdjuster {
                bounds: vec![1636.0, 5276.0],
                rates: vec![0.9, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 34787.8,
    }
}
