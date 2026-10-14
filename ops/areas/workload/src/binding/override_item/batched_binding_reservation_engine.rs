use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched binding reservation calculation.
pub trait BatchedBindingReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBindingReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedBindingReservationAdjuster {
    pub minimum: f64,
}

impl BatchedBindingReservationModifier for MinimumBatchedBindingReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedBindingReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedBindingReservationAdjuster {
    pub percent: f64,
}

impl BatchedBindingReservationModifier for PercentBatchedBindingReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedBindingReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitBatchedBindingReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedBindingReservationModifier for UnitBatchedBindingReservationAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &BatchedBindingReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedBindingReservationAdjuster {
    pub cap: f64,
}

impl BatchedBindingReservationModifier for CapBatchedBindingReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedBindingReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedBindingReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedBindingReservationModifier for TieredBatchedBindingReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedBindingReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched binding reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBindingReservationEngine {
    pub steps: Vec<Rc<dyn BatchedBindingReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBindingReservationEngine {
    pub fn run(&mut self, item: &BatchedBindingReservation) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedBindingReservationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBindingReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_binding_reservation_engine() -> BatchedBindingReservationEngine {
    BatchedBindingReservationEngine {
        steps: vec![
            Rc::new(MinimumBatchedBindingReservationAdjuster { minimum: 25.0 })
                as Rc<dyn BatchedBindingReservationModifier>,
            Rc::new(PercentBatchedBindingReservationAdjuster { percent: 15.9 }),
            Rc::new(UnitBatchedBindingReservationAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.07),
                    ("east".to_string(), 1.07),
                    ("north".to_string(), 1.02),
                ]),
            }),
            Rc::new(CapBatchedBindingReservationAdjuster { cap: 46889.0 }),
            Rc::new(TieredBatchedBindingReservationAdjuster {
                bounds: vec![1582.0, 2627.0, 5122.0],
                rates: vec![0.81, 0.92, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 20747.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_binding_reservation_engine();
        let item = new_batched_binding_reservation("code-606", "label-590");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
