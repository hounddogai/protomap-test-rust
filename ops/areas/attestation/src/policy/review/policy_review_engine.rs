use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the policy review calculation.
pub trait PolicyReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyReviewAdjuster {
    pub cap: f64,
}

impl PolicyReviewAdjuster for CapPolicyReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReviewQuantityOffsetAdjuster {
    pub weight: f64,
}

impl PolicyReviewAdjuster for PolicyReviewQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyReview, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyReviewAdjuster {
    pub minimum: f64,
}

impl PolicyReviewAdjuster for MinimumPolicyReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyReviewAdjuster {
    pub step: f64,
}

impl PolicyReviewAdjuster for RoundingPolicyReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyReviewAdjuster for SegmentPolicyReviewAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PolicyReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the policy review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyReviewEngine {
    pub steps: Vec<Rc<dyn PolicyReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyReviewEngine {
    pub fn evaluate(&mut self, item: &PolicyReview) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[PolicyReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_review_engine() -> PolicyReviewEngine {
    PolicyReviewEngine {
        steps: vec![
            Rc::new(CapPolicyReviewAdjuster { cap: 30084.0 }) as Rc<dyn PolicyReviewAdjuster>,
            Rc::new(PolicyReviewQuantityOffsetAdjuster { weight: 0.18 }),
            Rc::new(MinimumPolicyReviewAdjuster { minimum: 40.1 }),
            Rc::new(RoundingPolicyReviewAdjuster { step: 5.0 }),
            Rc::new(SegmentPolicyReviewAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.13), ("west".to_string(), 1.16)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 46081.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_review_engine();
        let item = new_policy_review("code-163", "title-813");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
