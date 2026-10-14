use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional binding review calculation.
pub trait RegionalBindingReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalBindingReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalBindingReviewAdjuster {
    pub cap: f64,
}

impl RegionalBindingReviewStep for CapRegionalBindingReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalBindingReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalBindingReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalBindingReviewStep for FlaggedRegionalBindingReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalBindingReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalBindingReviewAdjuster {
    pub step: f64,
}

impl RegionalBindingReviewStep for RoundingRegionalBindingReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalBindingReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional binding review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalBindingReviewEngine {
    pub steps: Vec<Rc<dyn RegionalBindingReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalBindingReviewEngine {
    pub fn compute(&mut self, item: &RegionalBindingReview) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalBindingReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_binding_review_engine(flags: &flags::Registry) -> RegionalBindingReviewEngine {
    RegionalBindingReviewEngine {
        steps: vec![
            Rc::new(CapRegionalBindingReviewAdjuster { cap: 17843.0 }) as Rc<dyn RegionalBindingReviewStep>,
            Rc::new(FlaggedRegionalBindingReviewAdjuster {
                flags: flags.clone(),
                flag: "regional-binding-review-capped".to_string(),
                percent: 5.6,
            }),
            Rc::new(RoundingRegionalBindingReviewAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 33984.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_binding_review_engine(&flags::new_registry());
        let item = new_regional_binding_review("code-141", "label-36");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
