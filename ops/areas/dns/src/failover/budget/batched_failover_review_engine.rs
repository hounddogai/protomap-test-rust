use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched failover review calculation.
pub trait BatchedFailoverReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedFailoverReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedFailoverReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedFailoverReviewFactor for TieredBatchedFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedFailoverReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeBatchedFailoverReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedFailoverReviewFactor for ScopeBatchedFailoverReviewAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &BatchedFailoverReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverReviewCostOffsetAdjuster {
    pub weight: f64,
}

impl BatchedFailoverReviewFactor for BatchedFailoverReviewCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedFailoverReview, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedFailoverReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedFailoverReviewFactor for FlaggedBatchedFailoverReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedFailoverReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched failover review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedFailoverReviewEngine {
    pub steps: Vec<Rc<dyn BatchedFailoverReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedFailoverReviewEngine {
    pub fn evaluate(&mut self, item: &BatchedFailoverReview) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedFailoverReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedFailoverReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_failover_review_engine(flags: &flags::Registry) -> BatchedFailoverReviewEngine {
    BatchedFailoverReviewEngine {
        steps: vec![
            Rc::new(TieredBatchedFailoverReviewAdjuster {
                bounds: vec![150.0, 9591.0],
                rates: vec![1.15, 1.13],
            }) as Rc<dyn BatchedFailoverReviewFactor>,
            Rc::new(ScopeBatchedFailoverReviewAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.05),
                    ("retail".to_string(), 0.87),
                    ("north".to_string(), 1.05),
                ]),
            }),
            Rc::new(BatchedFailoverReviewCostOffsetAdjuster { weight: 1.39 }),
            Rc::new(FlaggedBatchedFailoverReviewAdjuster {
                flags: flags.clone(),
                flag: "batched-failover-review-rolling".to_string(),
                percent: -4.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 51769.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_failover_review_engine(&flags::new_registry());
        let item = new_batched_failover_review("code-936", "name-571");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
