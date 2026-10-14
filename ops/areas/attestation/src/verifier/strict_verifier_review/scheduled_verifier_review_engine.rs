use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled verifier review calculation.
pub trait ScheduledVerifierReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledVerifierReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledVerifierReviewAdjuster {
    pub percent: f64,
}

impl ScheduledVerifierReviewModifier for PercentScheduledVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledVerifierReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledVerifierReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledVerifierReviewModifier for TieredScheduledVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledVerifierReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledVerifierReviewAdjuster {
    pub cap: f64,
}

impl ScheduledVerifierReviewModifier for CapScheduledVerifierReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledVerifierReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled verifier review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledVerifierReviewEngine {
    pub steps: Vec<Rc<dyn ScheduledVerifierReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledVerifierReviewEngine {
    pub fn price(&mut self, item: &ScheduledVerifierReview) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledVerifierReviewModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledVerifierReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_verifier_review_engine() -> ScheduledVerifierReviewEngine {
    ScheduledVerifierReviewEngine {
        steps: vec![
            Rc::new(PercentScheduledVerifierReviewAdjuster { percent: -0.9 })
                as Rc<dyn ScheduledVerifierReviewModifier>,
            Rc::new(TieredScheduledVerifierReviewAdjuster {
                bounds: vec![4547.0, 5797.0, 5814.0],
                rates: vec![1.14, 1.06, 1.13],
            }),
            Rc::new(CapScheduledVerifierReviewAdjuster { cap: 19426.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 78919.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_verifier_review_engine();
        let item = new_scheduled_verifier_review("code-60", "label-741");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
