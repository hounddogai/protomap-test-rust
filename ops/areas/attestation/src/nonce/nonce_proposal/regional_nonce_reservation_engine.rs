use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional nonce reservation calculation.
pub trait RegionalNonceReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalNonceReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservationFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RegionalNonceReservationStep for RegionalNonceReservationFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalNonceReservation, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalNonceReservationAdjuster {
    pub cap: f64,
}

impl RegionalNonceReservationStep for CapRegionalNonceReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalNonceReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalNonceReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalNonceReservationStep for TieredRegionalNonceReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalNonceReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalNonceReservationAdjuster {
    pub percent: f64,
}

impl RegionalNonceReservationStep for PercentRegionalNonceReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalNonceReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional nonce reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalNonceReservationEngine {
    pub steps: Vec<Rc<dyn RegionalNonceReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalNonceReservationEngine {
    pub fn compute(&mut self, item: &RegionalNonceReservation) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalNonceReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_nonce_reservation_engine() -> RegionalNonceReservationEngine {
    RegionalNonceReservationEngine {
        steps: vec![
            Rc::new(RegionalNonceReservationFloorValueOffsetAdjuster { weight: 0.78 })
                as Rc<dyn RegionalNonceReservationStep>,
            Rc::new(CapRegionalNonceReservationAdjuster { cap: 10269.0 }),
            Rc::new(TieredRegionalNonceReservationAdjuster {
                bounds: vec![1903.0, 8884.0],
                rates: vec![1.17, 0.96],
            }),
            Rc::new(PercentRegionalNonceReservationAdjuster { percent: 1.5 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 30128.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_nonce_reservation_engine();
        let item = new_regional_nonce_reservation("code-707", "title-249");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
