use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the query review calculation.
pub trait QueryReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryReviewAdjuster {
    pub minimum: f64,
}

impl QueryReviewAdjuster for MinimumQueryReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryReviewAdjuster for TieredQueryReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryReviewAdjuster for FlaggedQueryReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryReviewEngine {
    pub steps: Vec<Rc<dyn QueryReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryReviewEngine {
    pub fn compute(&mut self, item: &QueryReview) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_review_engine(flags: &flags::Registry) -> QueryReviewEngine {
    QueryReviewEngine {
        steps: vec![
            Rc::new(MinimumQueryReviewAdjuster { minimum: 0.3 }) as Rc<dyn QueryReviewAdjuster>,
            Rc::new(TieredQueryReviewAdjuster {
                bounds: vec![284.0, 1146.0, 8593.0],
                rates: vec![1.08, 1.0, 1.16],
            }),
            Rc::new(FlaggedQueryReviewAdjuster {
                flags: flags.clone(),
                flag: "query-review-legacy".to_string(),
                percent: 2.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 9821.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_review_engine(&flags::new_registry());
        let item = new_query_review("code-583", "title-45");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
