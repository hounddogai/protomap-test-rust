use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict grant reservation calculation.
pub trait StrictGrantReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictGrantReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictGrantReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictGrantReservationStep for FlaggedStrictGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictGrantReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictGrantReservationAdjuster {
    pub minimum: f64,
}

impl StrictGrantReservationStep for MinimumStrictGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictGrantReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictGrantReservationAdjuster {
    pub step: f64,
}

impl StrictGrantReservationStep for RoundingStrictGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictGrantReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictGrantReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictGrantReservationStep for TieredStrictGrantReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictGrantReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantReservationCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl StrictGrantReservationStep for StrictGrantReservationCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &StrictGrantReservation, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the strict grant reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictGrantReservationEngine {
    pub steps: Vec<Rc<dyn StrictGrantReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictGrantReservationEngine {
    pub fn compute(&mut self, item: &StrictGrantReservation) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[StrictGrantReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_grant_reservation_engine(flags: &flags::Registry) -> StrictGrantReservationEngine {
    StrictGrantReservationEngine {
        steps: vec![
            Rc::new(FlaggedStrictGrantReservationAdjuster {
                flags: flags.clone(),
                flag: "strict-grant-reservation-strict".to_string(),
                percent: -5.4,
            }) as Rc<dyn StrictGrantReservationStep>,
            Rc::new(MinimumStrictGrantReservationAdjuster { minimum: 23.7 }),
            Rc::new(RoundingStrictGrantReservationAdjuster { step: 0.5 }),
            Rc::new(TieredStrictGrantReservationAdjuster {
                bounds: vec![742.0, 2555.0, 3261.0, 4101.0],
                rates: vec![0.87, 1.02, 1.04, 0.97],
            }),
            Rc::new(StrictGrantReservationCreatedAtOffsetAdjuster { weight: 1.96 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 1196.6,
    }
}
