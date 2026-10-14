use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event review calculation.
pub trait AuditEventReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventReviewAdjuster {
    pub cap: f64,
}

impl AuditEventReviewFactor for CapAuditEventReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventReviewAdjuster {
    pub step: f64,
}

impl AuditEventReviewFactor for RoundingAuditEventReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReviewMaximumOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventReviewFactor for AuditEventReviewMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventReview, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the audit event review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventReviewEngine {
    pub steps: Vec<Rc<dyn AuditEventReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventReviewEngine {
    pub fn compute(&mut self, item: &AuditEventReview) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventReviewFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_review_engine() -> AuditEventReviewEngine {
    AuditEventReviewEngine {
        steps: vec![
            Rc::new(CapAuditEventReviewAdjuster { cap: 12565.0 }) as Rc<dyn AuditEventReviewFactor>,
            Rc::new(RoundingAuditEventReviewAdjuster { step: 0.05 }),
            Rc::new(AuditEventReviewMaximumOffsetAdjuster { weight: 1.47 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 22624.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_review_engine();
        let item = new_audit_event_review("code-616", "name-672");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
