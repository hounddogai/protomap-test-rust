use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted credential reservation calculation.
pub trait WeightedCredentialReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedCredentialReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedCredentialReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedCredentialReservationAdjuster for TieredWeightedCredentialReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedCredentialReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedCredentialReservationAdjuster {
    pub minimum: f64,
}

impl WeightedCredentialReservationAdjuster for MinimumWeightedCredentialReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedCredentialReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedCredentialReservationAdjuster {
    pub percent: f64,
}

impl WeightedCredentialReservationAdjuster for PercentWeightedCredentialReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedCredentialReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialReservationQuantityOffsetAdjuster {
    pub weight: f64,
}

impl WeightedCredentialReservationAdjuster for WeightedCredentialReservationQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedCredentialReservation, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedCredentialReservationAdjuster {
    pub cap: f64,
}

impl WeightedCredentialReservationAdjuster for CapWeightedCredentialReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedCredentialReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedCredentialReservationAdjuster {
    pub step: f64,
}

impl WeightedCredentialReservationAdjuster for RoundingWeightedCredentialReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedCredentialReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted credential reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedCredentialReservationEngine {
    pub steps: Vec<Rc<dyn WeightedCredentialReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedCredentialReservationEngine {
    pub fn evaluate(&mut self, item: &WeightedCredentialReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedCredentialReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_credential_reservation_engine() -> WeightedCredentialReservationEngine {
    WeightedCredentialReservationEngine {
        steps: vec![
            Rc::new(TieredWeightedCredentialReservationAdjuster {
                bounds: vec![436.0, 5250.0, 6834.0, 8282.0],
                rates: vec![1.03, 1.02, 0.99, 0.88],
            }) as Rc<dyn WeightedCredentialReservationAdjuster>,
            Rc::new(MinimumWeightedCredentialReservationAdjuster { minimum: 21.5 }),
            Rc::new(PercentWeightedCredentialReservationAdjuster { percent: 23.7 }),
            Rc::new(WeightedCredentialReservationQuantityOffsetAdjuster { weight: 0.38 }),
            Rc::new(CapWeightedCredentialReservationAdjuster { cap: 27315.0 }),
            Rc::new(RoundingWeightedCredentialReservationAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 53330.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_credential_reservation_engine();
        let item = new_weighted_credential_reservation("code-644", "title-516");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
