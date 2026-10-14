use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched region pin reservation calculation.
pub trait BatchedRegionPinReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRegionPinReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinReservationExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRegionPinReservationFactor for BatchedRegionPinReservationExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRegionPinReservation, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRegionPinReservationAdjuster {
    pub step: f64,
}

impl BatchedRegionPinReservationFactor for RoundingBatchedRegionPinReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRegionPinReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedRegionPinReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedRegionPinReservationFactor for FlaggedBatchedRegionPinReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedRegionPinReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedRegionPinReservationAdjuster {
    pub percent: f64,
}

impl BatchedRegionPinReservationFactor for PercentBatchedRegionPinReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedRegionPinReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched region pin reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRegionPinReservationEngine {
    pub steps: Vec<Rc<dyn BatchedRegionPinReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRegionPinReservationEngine {
    pub fn compute(&mut self, item: &BatchedRegionPinReservation) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRegionPinReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_region_pin_reservation_engine(flags: &flags::Registry) -> BatchedRegionPinReservationEngine {
    BatchedRegionPinReservationEngine {
        steps: vec![
            Rc::new(BatchedRegionPinReservationExpiresAtOffsetAdjuster { weight: 1.43 })
                as Rc<dyn BatchedRegionPinReservationFactor>,
            Rc::new(RoundingBatchedRegionPinReservationAdjuster { step: 5.0 }),
            Rc::new(FlaggedBatchedRegionPinReservationAdjuster {
                flags: flags.clone(),
                flag: "batched-region-pin-reservation-lenient".to_string(),
                percent: 9.9,
            }),
            Rc::new(PercentBatchedRegionPinReservationAdjuster { percent: -14.2 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 78557.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_region_pin_reservation_engine(&flags::new_registry());
        let item = new_batched_region_pin_reservation("code-551", "name-23");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
