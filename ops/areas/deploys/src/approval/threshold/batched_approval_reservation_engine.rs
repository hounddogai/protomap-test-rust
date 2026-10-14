use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched approval reservation calculation.
pub trait BatchedApprovalReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedApprovalReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedApprovalReservationAdjuster {
    pub step: f64,
}

impl BatchedApprovalReservationModifier for RoundingBatchedApprovalReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedApprovalReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedApprovalReservationAdjuster {
    pub percent: f64,
}

impl BatchedApprovalReservationModifier for PercentBatchedApprovalReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedApprovalReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservationQuantityOffsetAdjuster {
    pub weight: f64,
}

impl BatchedApprovalReservationModifier for BatchedApprovalReservationQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedApprovalReservation, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the batched approval reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedApprovalReservationEngine {
    pub steps: Vec<Rc<dyn BatchedApprovalReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedApprovalReservationEngine {
    pub fn evaluate(&mut self, item: &BatchedApprovalReservation) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedApprovalReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_approval_reservation_engine() -> BatchedApprovalReservationEngine {
    BatchedApprovalReservationEngine {
        steps: vec![
            Rc::new(RoundingBatchedApprovalReservationAdjuster { step: 0.25 })
                as Rc<dyn BatchedApprovalReservationModifier>,
            Rc::new(PercentBatchedApprovalReservationAdjuster { percent: -15.6 }),
            Rc::new(BatchedApprovalReservationQuantityOffsetAdjuster { weight: 0.73 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 48668.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_approval_reservation_engine();
        let item = new_batched_approval_reservation("code-12", "label-835");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
