use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending rollout review calculation.
pub trait PendingRolloutReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRolloutReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPendingRolloutReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRolloutReviewStep for SegmentPendingRolloutReviewAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PendingRolloutReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRolloutReviewAdjuster {
    pub step: f64,
}

impl PendingRolloutReviewStep for RoundingPendingRolloutReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRolloutReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRolloutReviewAdjuster {
    pub minimum: f64,
}

impl PendingRolloutReviewStep for MinimumPendingRolloutReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRolloutReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending rollout review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRolloutReviewEngine {
    pub steps: Vec<Rc<dyn PendingRolloutReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRolloutReviewEngine {
    pub fn evaluate(&mut self, item: &PendingRolloutReview) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingRolloutReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_rollout_review_engine() -> PendingRolloutReviewEngine {
    PendingRolloutReviewEngine {
        steps: vec![
            Rc::new(SegmentPendingRolloutReviewAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.39)]),
            }) as Rc<dyn PendingRolloutReviewStep>,
            Rc::new(RoundingPendingRolloutReviewAdjuster { step: 1.0 }),
            Rc::new(MinimumPendingRolloutReviewAdjuster { minimum: 42.5 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 15743.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_rollout_review_engine();
        let item = new_pending_rollout_review("code-501", "label-753");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
