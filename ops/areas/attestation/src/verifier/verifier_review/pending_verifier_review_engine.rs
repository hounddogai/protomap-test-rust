use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending verifier review calculation.
pub trait PendingVerifierReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingVerifierReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingVerifierReviewAdjuster {
    pub minimum: f64,
}

impl PendingVerifierReviewAdjuster for MinimumPendingVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingVerifierReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPendingVerifierReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingVerifierReviewAdjuster for UnitPendingVerifierReviewAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PendingVerifierReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingVerifierReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingVerifierReviewAdjuster for TieredPendingVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingVerifierReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingVerifierReviewAdjuster {
    pub percent: f64,
}

impl PendingVerifierReviewAdjuster for PercentPendingVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingVerifierReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending verifier review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingVerifierReviewEngine {
    pub steps: Vec<Rc<dyn PendingVerifierReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingVerifierReviewEngine {
    pub fn run(&mut self, item: &PendingVerifierReview) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[PendingVerifierReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_verifier_review_engine() -> PendingVerifierReviewEngine {
    PendingVerifierReviewEngine {
        steps: vec![
            Rc::new(MinimumPendingVerifierReviewAdjuster { minimum: 3.9 }) as Rc<dyn PendingVerifierReviewAdjuster>,
            Rc::new(UnitPendingVerifierReviewAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.01)]),
            }),
            Rc::new(TieredPendingVerifierReviewAdjuster {
                bounds: vec![931.0, 2416.0, 3863.0],
                rates: vec![1.03, 0.85, 1.01],
            }),
            Rc::new(PercentPendingVerifierReviewAdjuster { percent: 4.2 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 41473.3,
    }
}
