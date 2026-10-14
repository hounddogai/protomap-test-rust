use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual page review calculation.
pub trait ManualPageReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualPageReview, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualPageReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualPageReviewFactor for FlaggedManualPageReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualPageReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualPageReviewAdjuster {
    pub percent: f64,
}

impl ManualPageReviewFactor for PercentManualPageReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualPageReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageReviewVersionOffsetAdjuster {
    pub weight: f64,
}

impl ManualPageReviewFactor for ManualPageReviewVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ManualPageReview, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the manual page review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualPageReviewEngine {
    pub steps: Vec<Rc<dyn ManualPageReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualPageReviewEngine {
    pub fn evaluate(&mut self, item: &ManualPageReview) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualPageReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_page_review_engine(flags: &flags::Registry) -> ManualPageReviewEngine {
    ManualPageReviewEngine {
        steps: vec![
            Rc::new(FlaggedManualPageReviewAdjuster {
                flags: flags.clone(),
                flag: "manual-page-review-scheduled".to_string(),
                percent: -8.2,
            }) as Rc<dyn ManualPageReviewFactor>,
            Rc::new(PercentManualPageReviewAdjuster { percent: -8.2 }),
            Rc::new(ManualPageReviewVersionOffsetAdjuster { weight: 0.02 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 3366.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_page_review_engine(&flags::new_registry());
        let item = new_manual_page_review("code-365", "title-74");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
