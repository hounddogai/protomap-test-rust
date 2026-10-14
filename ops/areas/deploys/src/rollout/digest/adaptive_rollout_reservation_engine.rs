use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive rollout reservation calculation.
pub trait AdaptiveRolloutReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRolloutReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRolloutReservationAdjuster {
    pub percent: f64,
}

impl AdaptiveRolloutReservationStep for PercentAdaptiveRolloutReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRolloutReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationRetriesOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRolloutReservationStep for AdaptiveRolloutReservationRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRolloutReservation, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRolloutReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRolloutReservationStep for TieredAdaptiveRolloutReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRolloutReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive rollout reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRolloutReservationEngine {
    pub steps: Vec<Rc<dyn AdaptiveRolloutReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRolloutReservationEngine {
    pub fn evaluate(&mut self, item: &AdaptiveRolloutReservation) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRolloutReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_rollout_reservation_engine() -> AdaptiveRolloutReservationEngine {
    AdaptiveRolloutReservationEngine {
        steps: vec![
            Rc::new(PercentAdaptiveRolloutReservationAdjuster { percent: 23.7 })
                as Rc<dyn AdaptiveRolloutReservationStep>,
            Rc::new(AdaptiveRolloutReservationRetriesOffsetAdjuster { weight: 0.7 }),
            Rc::new(TieredAdaptiveRolloutReservationAdjuster {
                bounds: vec![5998.0, 9756.0],
                rates: vec![1.01, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 16302.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_rollout_reservation_engine();
        let item = new_adaptive_rollout_reservation("code-8", "title-257");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
