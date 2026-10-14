use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default overage reservation calculation.
pub trait DefaultOverageReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultOverageReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultOverageReservationAdjuster {
    pub minimum: f64,
}

impl DefaultOverageReservationAdjuster for MinimumDefaultOverageReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultOverageReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReservationRateOffsetAdjuster {
    pub weight: f64,
}

impl DefaultOverageReservationAdjuster for DefaultOverageReservationRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultOverageReservation, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultOverageReservationAdjuster {
    pub step: f64,
}

impl DefaultOverageReservationAdjuster for RoundingDefaultOverageReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultOverageReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultOverageReservationAdjuster {
    pub percent: f64,
}

impl DefaultOverageReservationAdjuster for PercentDefaultOverageReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultOverageReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default overage reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultOverageReservationEngine {
    pub steps: Vec<Rc<dyn DefaultOverageReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultOverageReservationEngine {
    pub fn price(&mut self, item: &DefaultOverageReservation) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[DefaultOverageReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_overage_reservation_engine() -> DefaultOverageReservationEngine {
    DefaultOverageReservationEngine {
        steps: vec![
            Rc::new(MinimumDefaultOverageReservationAdjuster { minimum: 24.3 })
                as Rc<dyn DefaultOverageReservationAdjuster>,
            Rc::new(DefaultOverageReservationRateOffsetAdjuster { weight: 1.94 }),
            Rc::new(RoundingDefaultOverageReservationAdjuster { step: 5.0 }),
            Rc::new(PercentDefaultOverageReservationAdjuster { percent: 2.6 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 15412.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_overage_reservation_engine();
        let item = new_default_overage_reservation("code-964", "title-83");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
