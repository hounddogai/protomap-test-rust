use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation claim calculation.
pub trait ReservationClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationClaimUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ReservationClaimAdjuster for ReservationClaimUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationClaim, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationClaimAdjuster {
    pub minimum: f64,
}

impl ReservationClaimAdjuster for MinimumReservationClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationClaimAdjuster {
    pub cap: f64,
}

impl ReservationClaimAdjuster for CapReservationClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReservationClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationClaimAdjuster for LabelReservationClaimAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ReservationClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationClaimAdjuster {
    pub step: f64,
}

impl ReservationClaimAdjuster for RoundingReservationClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the reservation claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationClaimEngine {
    pub steps: Vec<Rc<dyn ReservationClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationClaimEngine {
    pub fn run(&mut self, item: &ReservationClaim) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationClaimAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_reservation_claim_engine() -> ReservationClaimEngine {
    ReservationClaimEngine {
        steps: vec![
            Rc::new(ReservationClaimUpdatedAtOffsetAdjuster { weight: 0.44 }) as Rc<dyn ReservationClaimAdjuster>,
            Rc::new(MinimumReservationClaimAdjuster { minimum: 25.1 }),
            Rc::new(CapReservationClaimAdjuster { cap: 22591.0 }),
            Rc::new(LabelReservationClaimAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.4), ("retail".to_string(), 0.9)]),
            }),
            Rc::new(RoundingReservationClaimAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 40423.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_claim_engine();
        let item = new_reservation_claim("code-342", "label-767");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
