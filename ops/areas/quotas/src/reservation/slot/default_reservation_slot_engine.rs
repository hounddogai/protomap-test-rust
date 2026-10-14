use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default reservation slot calculation.
pub trait DefaultReservationSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultReservationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDefaultReservationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultReservationSlotFactor for NamespaceDefaultReservationSlotAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &DefaultReservationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultReservationSlotAdjuster {
    pub minimum: f64,
}

impl DefaultReservationSlotFactor for MinimumDefaultReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultReservationSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultReservationSlotAdjuster {
    pub percent: f64,
}

impl DefaultReservationSlotFactor for PercentDefaultReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultReservationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationSlotCostOffsetAdjuster {
    pub weight: f64,
}

impl DefaultReservationSlotFactor for DefaultReservationSlotCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultReservationSlot, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultReservationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultReservationSlotFactor for TieredDefaultReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultReservationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the default reservation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultReservationSlotEngine {
    pub steps: Vec<Rc<dyn DefaultReservationSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultReservationSlotEngine {
    pub fn compute(&mut self, item: &DefaultReservationSlot) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultReservationSlotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultReservationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_reservation_slot_engine() -> DefaultReservationSlotEngine {
    DefaultReservationSlotEngine {
        steps: vec![
            Rc::new(NamespaceDefaultReservationSlotAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.93), ("online".to_string(), 1.33)]),
            }) as Rc<dyn DefaultReservationSlotFactor>,
            Rc::new(MinimumDefaultReservationSlotAdjuster { minimum: 12.0 }),
            Rc::new(PercentDefaultReservationSlotAdjuster { percent: 15.4 }),
            Rc::new(DefaultReservationSlotCostOffsetAdjuster { weight: 0.69 }),
            Rc::new(TieredDefaultReservationSlotAdjuster {
                bounds: vec![5401.0, 8345.0],
                rates: vec![0.88, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 39562.5,
    }
}
