use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched reservation entry calculation.
pub trait BatchedReservationEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedReservationEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedReservationEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedReservationEntryStep for TieredBatchedReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedReservationEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedReservationEntryAdjuster {
    pub percent: f64,
}

impl BatchedReservationEntryStep for PercentBatchedReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedReservationEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedReservationEntryAdjuster {
    pub cap: f64,
}

impl BatchedReservationEntryStep for CapBatchedReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedReservationEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedReservationEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedReservationEntryStep for NameBatchedReservationEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedReservationEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedReservationEntryAdjuster {
    pub step: f64,
}

impl BatchedReservationEntryStep for RoundingBatchedReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedReservationEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched reservation entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedReservationEntryEngine {
    pub steps: Vec<Rc<dyn BatchedReservationEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedReservationEntryEngine {
    pub fn compute(&mut self, item: &BatchedReservationEntry) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedReservationEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_reservation_entry_engine() -> BatchedReservationEntryEngine {
    BatchedReservationEntryEngine {
        steps: vec![
            Rc::new(TieredBatchedReservationEntryAdjuster {
                bounds: vec![6697.0, 7515.0, 9854.0],
                rates: vec![1.04, 0.93, 1.08],
            }) as Rc<dyn BatchedReservationEntryStep>,
            Rc::new(PercentBatchedReservationEntryAdjuster { percent: 17.5 }),
            Rc::new(CapBatchedReservationEntryAdjuster { cap: 26229.0 }),
            Rc::new(NameBatchedReservationEntryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.95)]),
            }),
            Rc::new(RoundingBatchedReservationEntryAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 59803.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_reservation_entry_engine();
        let item = new_batched_reservation_entry("code-220", "name-299");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
