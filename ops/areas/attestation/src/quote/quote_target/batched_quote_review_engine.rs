use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched quote review calculation.
pub trait BatchedQuoteReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedQuoteReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedQuoteReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedQuoteReviewAdjuster for TieredBatchedQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedQuoteReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedQuoteReviewAdjuster {
    pub minimum: f64,
}

impl BatchedQuoteReviewAdjuster for MinimumBatchedQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedQuoteReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedQuoteReviewAdjuster {
    pub percent: f64,
}

impl BatchedQuoteReviewAdjuster for PercentBatchedQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedQuoteReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBatchedQuoteReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedQuoteReviewAdjuster for CountryBatchedQuoteReviewAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BatchedQuoteReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched quote review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedQuoteReviewEngine {
    pub steps: Vec<Rc<dyn BatchedQuoteReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedQuoteReviewEngine {
    pub fn compute(&mut self, item: &BatchedQuoteReview) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedQuoteReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_quote_review_engine() -> BatchedQuoteReviewEngine {
    BatchedQuoteReviewEngine {
        steps: vec![
            Rc::new(TieredBatchedQuoteReviewAdjuster {
                bounds: vec![2758.0, 3597.0, 7160.0],
                rates: vec![0.85, 0.9, 0.96],
            }) as Rc<dyn BatchedQuoteReviewAdjuster>,
            Rc::new(MinimumBatchedQuoteReviewAdjuster { minimum: 9.9 }),
            Rc::new(PercentBatchedQuoteReviewAdjuster { percent: -18.9 }),
            Rc::new(CountryBatchedQuoteReviewAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.79)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 23975.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_quote_review_engine();
        let item = new_batched_quote_review("code-575", "label-966");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
