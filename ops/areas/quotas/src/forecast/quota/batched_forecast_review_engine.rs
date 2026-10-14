use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched forecast review calculation.
pub trait BatchedForecastReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedForecastReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedForecastReviewAdjuster {
    pub cap: f64,
}

impl BatchedForecastReviewAdjuster for CapBatchedForecastReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedForecastReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedForecastReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedForecastReviewAdjuster for FlaggedBatchedForecastReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedForecastReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedForecastReviewAdjuster {
    pub percent: f64,
}

impl BatchedForecastReviewAdjuster for PercentBatchedForecastReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedForecastReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedForecastReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedForecastReviewAdjuster for TieredBatchedForecastReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedForecastReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedForecastReviewAdjuster {
    pub minimum: f64,
}

impl BatchedForecastReviewAdjuster for MinimumBatchedForecastReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedForecastReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeBatchedForecastReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedForecastReviewAdjuster for ScopeBatchedForecastReviewAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &BatchedForecastReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched forecast review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedForecastReviewEngine {
    pub steps: Vec<Rc<dyn BatchedForecastReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedForecastReviewEngine {
    pub fn compute(&mut self, item: &BatchedForecastReview) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedForecastReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_forecast_review_engine(flags: &flags::Registry) -> BatchedForecastReviewEngine {
    BatchedForecastReviewEngine {
        steps: vec![
            Rc::new(CapBatchedForecastReviewAdjuster { cap: 29052.0 }) as Rc<dyn BatchedForecastReviewAdjuster>,
            Rc::new(FlaggedBatchedForecastReviewAdjuster {
                flags: flags.clone(),
                flag: "batched-forecast-review-weighted".to_string(),
                percent: -2.1,
            }),
            Rc::new(PercentBatchedForecastReviewAdjuster { percent: -14.6 }),
            Rc::new(TieredBatchedForecastReviewAdjuster {
                bounds: vec![2421.0, 5572.0],
                rates: vec![1.1, 0.92],
            }),
            Rc::new(MinimumBatchedForecastReviewAdjuster { minimum: 32.7 }),
            Rc::new(ScopeBatchedForecastReviewAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.36),
                    ("south".to_string(), 1.37),
                    ("online".to_string(), 0.8),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 52305.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_forecast_review_engine(&flags::new_registry());
        let item = new_batched_forecast_review("code-213", "name-61");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
