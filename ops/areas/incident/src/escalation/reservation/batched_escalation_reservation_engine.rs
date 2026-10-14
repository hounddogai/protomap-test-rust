use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched escalation reservation calculation.
pub trait BatchedEscalationReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedEscalationReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedEscalationReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedEscalationReservationModifier for TieredBatchedEscalationReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedEscalationReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedEscalationReservationAdjuster {
    pub cap: f64,
}

impl BatchedEscalationReservationModifier for CapBatchedEscalationReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedEscalationReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionBatchedEscalationReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedEscalationReservationModifier for RegionBatchedEscalationReservationAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &BatchedEscalationReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedEscalationReservationAdjuster {
    pub percent: f64,
}

impl BatchedEscalationReservationModifier for PercentBatchedEscalationReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedEscalationReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched escalation reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedEscalationReservationEngine {
    pub steps: Vec<Rc<dyn BatchedEscalationReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedEscalationReservationEngine {
    pub fn evaluate(&mut self, item: &BatchedEscalationReservation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedEscalationReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_escalation_reservation_engine() -> BatchedEscalationReservationEngine {
    BatchedEscalationReservationEngine {
        steps: vec![
            Rc::new(TieredBatchedEscalationReservationAdjuster {
                bounds: vec![2164.0, 4958.0, 9049.0],
                rates: vec![1.07, 0.85, 1.18],
            }) as Rc<dyn BatchedEscalationReservationModifier>,
            Rc::new(CapBatchedEscalationReservationAdjuster { cap: 16543.0 }),
            Rc::new(RegionBatchedEscalationReservationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.11), ("west".to_string(), 0.88)]),
            }),
            Rc::new(PercentBatchedEscalationReservationAdjuster { percent: 14.6 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 33813.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_escalation_reservation_engine();
        let item = new_batched_escalation_reservation("code-924", "name-938");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
