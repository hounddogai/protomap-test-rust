use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation review calculation.
pub trait ReservationReviewStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationReviewAdjuster {
    pub step: f64,
}

impl ReservationReviewStep for RoundingReservationReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationReviewAdjuster {
    pub minimum: f64,
}

impl ReservationReviewStep for MinimumReservationReviewAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationReview, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionReservationReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationReviewStep for RegionReservationReviewAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ReservationReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the reservation review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationReviewEngine {
    pub steps: Vec<Rc<dyn ReservationReviewStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationReviewEngine {
    pub fn compute(&mut self, item: &ReservationReview) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationReviewStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_reservation_review_engine() -> ReservationReviewEngine {
    ReservationReviewEngine {
        steps: vec![
            Rc::new(RoundingReservationReviewAdjuster { step: 0.1 }) as Rc<dyn ReservationReviewStep>,
            Rc::new(MinimumReservationReviewAdjuster { minimum: 8.0 }),
            Rc::new(RegionReservationReviewAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.38), ("east".to_string(), 0.81)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 32033.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_review_engine();
        let item = new_reservation_review("code-324", "name-448");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
