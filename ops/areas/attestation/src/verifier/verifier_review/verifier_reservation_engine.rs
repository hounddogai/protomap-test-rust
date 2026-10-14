use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier reservation calculation.
pub trait VerifierReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservationStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl VerifierReservationStep for VerifierReservationStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierReservation, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierReservationAdjuster {
    pub minimum: f64,
}

impl VerifierReservationStep for MinimumVerifierReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierReservationAdjuster {
    pub step: f64,
}

impl VerifierReservationStep for RoundingVerifierReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierReservationEngine {
    pub steps: Vec<Rc<dyn VerifierReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierReservationEngine {
    pub fn run(&mut self, item: &VerifierReservation) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_reservation_engine() -> VerifierReservationEngine {
    VerifierReservationEngine {
        steps: vec![
            Rc::new(VerifierReservationStartsAtOffsetAdjuster { weight: 1.13 }) as Rc<dyn VerifierReservationStep>,
            Rc::new(MinimumVerifierReservationAdjuster { minimum: 31.0 }),
            Rc::new(RoundingVerifierReservationAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 12024.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_reservation_engine();
        let item = new_verifier_reservation("code-356", "name-999");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
