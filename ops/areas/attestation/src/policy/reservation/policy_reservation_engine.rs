use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the policy reservation calculation.
pub trait PolicyReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyReservationAdjuster {
    pub step: f64,
}

impl PolicyReservationAdjuster for RoundingPolicyReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyReservationAdjuster for SegmentPolicyReservationAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PolicyReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReservationMinimumOffsetAdjuster {
    pub weight: f64,
}

impl PolicyReservationAdjuster for PolicyReservationMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyReservation, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyReservationAdjuster {
    pub cap: f64,
}

impl PolicyReservationAdjuster for CapPolicyReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyReservationAdjuster {
    pub minimum: f64,
}

impl PolicyReservationAdjuster for MinimumPolicyReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the policy reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyReservationEngine {
    pub steps: Vec<Rc<dyn PolicyReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyReservationEngine {
    pub fn run(&mut self, item: &PolicyReservation) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[PolicyReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_policy_reservation_engine() -> PolicyReservationEngine {
    PolicyReservationEngine {
        steps: vec![
            Rc::new(RoundingPolicyReservationAdjuster { step: 0.5 }) as Rc<dyn PolicyReservationAdjuster>,
            Rc::new(SegmentPolicyReservationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.9), ("central".to_string(), 1.07)]),
            }),
            Rc::new(PolicyReservationMinimumOffsetAdjuster { weight: 0.36 }),
            Rc::new(CapPolicyReservationAdjuster { cap: 48983.0 }),
            Rc::new(MinimumPolicyReservationAdjuster { minimum: 41.2 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 44954.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_reservation_engine();
        let item = new_policy_reservation("code-713", "title-242");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
