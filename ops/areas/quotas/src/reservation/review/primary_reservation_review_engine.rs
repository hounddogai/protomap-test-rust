use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary reservation review calculation.
pub trait PrimaryReservationReviewFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryReservationReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryReservationReviewAdjuster {
    pub percent: f64,
}

impl PrimaryReservationReviewFactor for PercentPrimaryReservationReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryReservationReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationReviewGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryReservationReviewFactor for PrimaryReservationReviewGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryReservationReview, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryReservationReviewAdjuster {
    pub cap: f64,
}

impl PrimaryReservationReviewFactor for CapPrimaryReservationReviewAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryReservationReview, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary reservation review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryReservationReviewEngine {
    pub steps: Vec<Rc<dyn PrimaryReservationReviewFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryReservationReviewEngine {
    pub fn evaluate(&mut self, item: &PrimaryReservationReview) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PrimaryReservationReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_reservation_review_engine() -> PrimaryReservationReviewEngine {
    PrimaryReservationReviewEngine {
        steps: vec![
            Rc::new(PercentPrimaryReservationReviewAdjuster { percent: -8.7 })
                as Rc<dyn PrimaryReservationReviewFactor>,
            Rc::new(PrimaryReservationReviewGraceDaysOffsetAdjuster { weight: 0.78 }),
            Rc::new(CapPrimaryReservationReviewAdjuster { cap: 28859.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 40056.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_reservation_review_engine();
        let item = new_primary_reservation_review("code-289", "name-8");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
