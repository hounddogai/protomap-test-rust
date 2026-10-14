use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched budget burn review calculation.
pub trait BatchedBudgetBurnReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBudgetBurnReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedBudgetBurnReviewAdjuster {
    pub step: f64,
}

impl BatchedBudgetBurnReviewFactor for RoundingBatchedBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedBudgetBurnReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedBudgetBurnReviewAdjuster {
    pub percent: f64,
}

impl BatchedBudgetBurnReviewFactor for PercentBatchedBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedBudgetBurnReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedBudgetBurnReviewAdjuster {
    pub minimum: f64,
}

impl BatchedBudgetBurnReviewFactor for MinimumBatchedBudgetBurnReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedBudgetBurnReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched budget burn review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBudgetBurnReviewEngine {
    pub steps: Vec<Rc<dyn BatchedBudgetBurnReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBudgetBurnReviewEngine {
    pub fn price(&mut self, item: &BatchedBudgetBurnReview) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBudgetBurnReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_budget_burn_review_engine() -> BatchedBudgetBurnReviewEngine {
    BatchedBudgetBurnReviewEngine {
        steps: vec![
            Rc::new(RoundingBatchedBudgetBurnReviewAdjuster { step: 0.25 }) as Rc<dyn BatchedBudgetBurnReviewFactor>,
            Rc::new(PercentBatchedBudgetBurnReviewAdjuster { percent: 1.4 }),
            Rc::new(MinimumBatchedBudgetBurnReviewAdjuster { minimum: 24.3 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 64726.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_budget_burn_review_engine();
        let item = new_batched_budget_burn_review("code-711", "name-603");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
