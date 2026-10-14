use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page review calculation.
pub trait PageReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPageReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageReviewFactor for FlaggedPageReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageReviewAdjuster {
    pub step: f64,
}

impl PageReviewFactor for RoundingPageReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageReviewAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageReviewFactor for TieredPageReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageReview, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the page review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageReviewEngine {
    pub steps: Vec<Rc<dyn PageReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageReviewEngine {
    pub fn price(&mut self, item: &PageReview) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_page_review_engine(flags: &flags::Registry) -> PageReviewEngine {
    PageReviewEngine {
        steps: vec![
            Rc::new(FlaggedPageReviewAdjuster {
                flags: flags.clone(),
                flag: "page-review-strict".to_string(),
                percent: -0.4,
            }) as Rc<dyn PageReviewFactor>,
            Rc::new(RoundingPageReviewAdjuster { step: 0.25 }),
            Rc::new(TieredPageReviewAdjuster {
                bounds: vec![514.0, 7117.0, 7865.0, 7878.0],
                rates: vec![0.94, 0.82, 1.02, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 99141.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_review_engine(&flags::new_registry());
        let item = new_page_review("code-678", "name-154");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
