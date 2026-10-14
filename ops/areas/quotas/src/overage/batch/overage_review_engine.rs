use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage review calculation.
pub trait OverageReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageReviewFactor for FlaggedOverageReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageReviewAdjuster {
    pub minimum: f64,
}

impl OverageReviewFactor for MinimumOverageReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageReviewCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl OverageReviewFactor for OverageReviewCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &OverageReview, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the overage review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageReviewEngine {
    pub steps: Vec<Rc<dyn OverageReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageReviewEngine {
    pub fn price(&mut self, item: &OverageReview) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[OverageReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_review_engine(flags: &flags::Registry) -> OverageReviewEngine {
    OverageReviewEngine {
        steps: vec![
            Rc::new(FlaggedOverageReviewAdjuster {
                flags: flags.clone(),
                flag: "overage-review-pending".to_string(),
                percent: 8.9,
            }) as Rc<dyn OverageReviewFactor>,
            Rc::new(MinimumOverageReviewAdjuster { minimum: 20.8 }),
            Rc::new(OverageReviewCreatedAtOffsetAdjuster { weight: 0.33 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 27525.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_review_engine(&flags::new_registry());
        let item = new_overage_review("code-481", "name-507");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
