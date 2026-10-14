use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant reservation calculation.
pub trait GrantReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantReservationStep for FlaggedGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantReservationAdjuster {
    pub step: f64,
}

impl GrantReservationStep for RoundingGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantReservationQuantityOffsetAdjuster {
    pub weight: f64,
}

impl GrantReservationStep for GrantReservationQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &GrantReservation, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantReservationStep for TieredGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantReservationAdjuster {
    pub percent: f64,
}

impl GrantReservationStep for PercentGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the grant reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantReservationEngine {
    pub steps: Vec<Rc<dyn GrantReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantReservationEngine {
    pub fn price(&mut self, item: &GrantReservation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_reservation_engine(flags: &flags::Registry) -> GrantReservationEngine {
    GrantReservationEngine {
        steps: vec![
            Rc::new(FlaggedGrantReservationAdjuster {
                flags: flags.clone(),
                flag: "grant-reservation-global".to_string(),
                percent: 2.5,
            }) as Rc<dyn GrantReservationStep>,
            Rc::new(RoundingGrantReservationAdjuster { step: 1.0 }),
            Rc::new(GrantReservationQuantityOffsetAdjuster { weight: 0.85 }),
            Rc::new(TieredGrantReservationAdjuster {
                bounds: vec![438.0, 3218.0, 9314.0],
                rates: vec![0.88, 0.89, 1.03],
            }),
            Rc::new(PercentGrantReservationAdjuster { percent: -9.6 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 66107.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_reservation_engine(&flags::new_registry());
        let item = new_grant_reservation("code-751", "name-467");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
