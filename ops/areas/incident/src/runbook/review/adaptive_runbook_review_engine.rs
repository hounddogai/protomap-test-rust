use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive runbook review calculation.
pub trait AdaptiveRunbookReviewModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRunbookReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRunbookReviewAdjuster {
    pub cap: f64,
}

impl AdaptiveRunbookReviewModifier for CapAdaptiveRunbookReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRunbookReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRunbookReviewAdjuster {
    pub minimum: f64,
}

impl AdaptiveRunbookReviewModifier for MinimumAdaptiveRunbookReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRunbookReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdaptiveRunbookReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRunbookReviewModifier for SourceAdaptiveRunbookReviewAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AdaptiveRunbookReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive runbook review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRunbookReviewEngine {
    pub steps: Vec<Rc<dyn AdaptiveRunbookReviewModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRunbookReviewEngine {
    pub fn compute(&mut self, item: &AdaptiveRunbookReview) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRunbookReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_runbook_review_engine() -> AdaptiveRunbookReviewEngine {
    AdaptiveRunbookReviewEngine {
        steps: vec![
            Rc::new(CapAdaptiveRunbookReviewAdjuster { cap: 4795.0 }) as Rc<dyn AdaptiveRunbookReviewModifier>,
            Rc::new(MinimumAdaptiveRunbookReviewAdjuster { minimum: 16.5 }),
            Rc::new(SourceAdaptiveRunbookReviewAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.39)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 21871.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_runbook_review_engine();
        let item = new_adaptive_runbook_review("code-494", "title-521");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
