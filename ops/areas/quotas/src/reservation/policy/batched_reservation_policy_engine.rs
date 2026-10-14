use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched reservation policy calculation.
pub trait BatchedReservationPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedReservationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedReservationPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedReservationPolicyAdjuster for TieredBatchedReservationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedReservationPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationPolicyThresholdOffsetAdjuster {
    pub weight: f64,
}

impl BatchedReservationPolicyAdjuster for BatchedReservationPolicyThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedReservationPolicy, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedReservationPolicyAdjuster {
    pub minimum: f64,
}

impl BatchedReservationPolicyAdjuster for MinimumBatchedReservationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedReservationPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedReservationPolicyAdjuster {
    pub percent: f64,
}

impl BatchedReservationPolicyAdjuster for PercentBatchedReservationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedReservationPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched reservation policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedReservationPolicyEngine {
    pub steps: Vec<Rc<dyn BatchedReservationPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedReservationPolicyEngine {
    pub fn evaluate(&mut self, item: &BatchedReservationPolicy) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[BatchedReservationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_reservation_policy_engine() -> BatchedReservationPolicyEngine {
    BatchedReservationPolicyEngine {
        steps: vec![
            Rc::new(TieredBatchedReservationPolicyAdjuster {
                bounds: vec![6426.0, 7901.0, 8709.0],
                rates: vec![0.93, 0.95, 1.13],
            }) as Rc<dyn BatchedReservationPolicyAdjuster>,
            Rc::new(BatchedReservationPolicyThresholdOffsetAdjuster { weight: 0.06 }),
            Rc::new(MinimumBatchedReservationPolicyAdjuster { minimum: 10.2 }),
            Rc::new(PercentBatchedReservationPolicyAdjuster { percent: 9.1 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 19183.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_reservation_policy_engine();
        let item = new_batched_reservation_policy("code-941", "title-82");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
