use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence review calculation.
pub trait EvidenceReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReviewGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceReviewStep for EvidenceReviewGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceReview, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceReviewAdjuster {
    pub cap: f64,
}

impl EvidenceReviewStep for CapEvidenceReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceReviewStep for TieredEvidenceReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceReviewAdjuster {
    pub step: f64,
}

impl EvidenceReviewStep for RoundingEvidenceReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the evidence review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceReviewEngine {
    pub steps: Vec<Rc<dyn EvidenceReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceReviewEngine {
    pub fn run(&mut self, item: &EvidenceReview) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_review_engine() -> EvidenceReviewEngine {
    EvidenceReviewEngine {
        steps: vec![
            Rc::new(EvidenceReviewGraceDaysOffsetAdjuster { weight: 0.39 }) as Rc<dyn EvidenceReviewStep>,
            Rc::new(CapEvidenceReviewAdjuster { cap: 48476.0 }),
            Rc::new(TieredEvidenceReviewAdjuster {
                bounds: vec![1145.0, 8431.0, 8578.0, 9973.0],
                rates: vec![1.1, 1.01, 0.97, 1.18],
            }),
            Rc::new(RoundingEvidenceReviewAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 63568.3,
    }
}
