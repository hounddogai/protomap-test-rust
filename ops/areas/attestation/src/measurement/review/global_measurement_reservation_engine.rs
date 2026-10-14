use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global measurement reservation calculation.
pub trait GlobalMeasurementReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalMeasurementReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalMeasurementReservationAdjuster {
    pub cap: f64,
}

impl GlobalMeasurementReservationFactor for CapGlobalMeasurementReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalMeasurementReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalMeasurementReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalMeasurementReservationFactor for TieredGlobalMeasurementReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalMeasurementReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalMeasurementReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalMeasurementReservationFactor for LabelGlobalMeasurementReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalMeasurementReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalMeasurementReservationAdjuster {
    pub step: f64,
}

impl GlobalMeasurementReservationFactor for RoundingGlobalMeasurementReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalMeasurementReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementReservationGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl GlobalMeasurementReservationFactor for GlobalMeasurementReservationGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalMeasurementReservation, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the global measurement reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalMeasurementReservationEngine {
    pub steps: Vec<Rc<dyn GlobalMeasurementReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalMeasurementReservationEngine {
    pub fn price(&mut self, item: &GlobalMeasurementReservation) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalMeasurementReservationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalMeasurementReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_measurement_reservation_engine() -> GlobalMeasurementReservationEngine {
    GlobalMeasurementReservationEngine {
        steps: vec![
            Rc::new(CapGlobalMeasurementReservationAdjuster { cap: 42618.0 })
                as Rc<dyn GlobalMeasurementReservationFactor>,
            Rc::new(TieredGlobalMeasurementReservationAdjuster {
                bounds: vec![120.0, 9489.0],
                rates: vec![0.85, 1.03],
            }),
            Rc::new(LabelGlobalMeasurementReservationAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.72), ("west".to_string(), 0.98)]),
            }),
            Rc::new(RoundingGlobalMeasurementReservationAdjuster { step: 0.1 }),
            Rc::new(GlobalMeasurementReservationGraceDaysOffsetAdjuster { weight: 0.64 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 32417.8,
    }
}
