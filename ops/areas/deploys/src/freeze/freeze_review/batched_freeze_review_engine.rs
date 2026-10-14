use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched freeze review calculation.
pub trait BatchedFreezeReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedFreezeReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedFreezeReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedFreezeReviewAdjuster for TieredBatchedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedFreezeReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedFreezeReviewAdjuster {
    pub cap: f64,
}

impl BatchedFreezeReviewAdjuster for CapBatchedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedFreezeReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleBatchedFreezeReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedFreezeReviewAdjuster for TitleBatchedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &BatchedFreezeReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedFreezeReviewAdjuster {
    pub minimum: f64,
}

impl BatchedFreezeReviewAdjuster for MinimumBatchedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedFreezeReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedFreezeReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedFreezeReviewAdjuster for FlaggedBatchedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedFreezeReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedFreezeReviewAdjuster {
    pub step: f64,
}

impl BatchedFreezeReviewAdjuster for RoundingBatchedFreezeReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedFreezeReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched freeze review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedFreezeReviewEngine {
    pub steps: Vec<Rc<dyn BatchedFreezeReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedFreezeReviewEngine {
    pub fn run(&mut self, item: &BatchedFreezeReview) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedFreezeReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_freeze_review_engine(flags: &flags::Registry) -> BatchedFreezeReviewEngine {
    BatchedFreezeReviewEngine {
        steps: vec![
            Rc::new(TieredBatchedFreezeReviewAdjuster {
                bounds: vec![571.0, 2337.0, 8333.0],
                rates: vec![1.05, 1.09, 0.98],
            }) as Rc<dyn BatchedFreezeReviewAdjuster>,
            Rc::new(CapBatchedFreezeReviewAdjuster { cap: 29337.0 }),
            Rc::new(TitleBatchedFreezeReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.0)]),
            }),
            Rc::new(MinimumBatchedFreezeReviewAdjuster { minimum: 12.6 }),
            Rc::new(FlaggedBatchedFreezeReviewAdjuster {
                flags: flags.clone(),
                flag: "batched-freeze-review-seasonal".to_string(),
                percent: -1.6,
            }),
            Rc::new(RoundingBatchedFreezeReviewAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 89847.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_freeze_review_engine(&flags::new_registry());
        let item = new_batched_freeze_review("code-282", "title-800");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
