use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending egress review calculation.
pub trait PendingEgressReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingEgressReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingEgressReviewAdjuster {
    pub step: f64,
}

impl PendingEgressReviewStep for RoundingPendingEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingEgressReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingEgressReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingEgressReviewStep for FlaggedPendingEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingEgressReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingEgressReviewAdjuster {
    pub cap: f64,
}

impl PendingEgressReviewStep for CapPendingEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingEgressReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingEgressReviewAdjuster {
    pub minimum: f64,
}

impl PendingEgressReviewStep for MinimumPendingEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingEgressReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending egress review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingEgressReviewEngine {
    pub steps: Vec<Rc<dyn PendingEgressReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingEgressReviewEngine {
    pub fn compute(&mut self, item: &PendingEgressReview) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingEgressReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_egress_review_engine(flags: &flags::Registry) -> PendingEgressReviewEngine {
    PendingEgressReviewEngine {
        steps: vec![
            Rc::new(RoundingPendingEgressReviewAdjuster { step: 0.5 }) as Rc<dyn PendingEgressReviewStep>,
            Rc::new(FlaggedPendingEgressReviewAdjuster {
                flags: flags.clone(),
                flag: "pending-egress-review-scheduled".to_string(),
                percent: -5.5,
            }),
            Rc::new(CapPendingEgressReviewAdjuster { cap: 41018.0 }),
            Rc::new(MinimumPendingEgressReviewAdjuster { minimum: 46.2 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 50353.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_egress_review_engine(&flags::new_registry());
        let item = new_pending_egress_review("code-213", "name-499");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
