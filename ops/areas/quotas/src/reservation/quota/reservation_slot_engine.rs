use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the reservation slot calculation.
pub trait ReservationSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameReservationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationSlotAdjuster for NameReservationSlotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ReservationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSlotFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl ReservationSlotAdjuster for ReservationSlotFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationSlot, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationSlotAdjuster {
    pub percent: f64,
}

impl ReservationSlotAdjuster for PercentReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationSlotAdjuster for FlaggedReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the reservation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationSlotEngine {
    pub steps: Vec<Rc<dyn ReservationSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationSlotEngine {
    pub fn evaluate(&mut self, item: &ReservationSlot) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationSlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_slot_engine(flags: &flags::Registry) -> ReservationSlotEngine {
    ReservationSlotEngine {
        steps: vec![
            Rc::new(NameReservationSlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.88)]),
            }) as Rc<dyn ReservationSlotAdjuster>,
            Rc::new(ReservationSlotFloorValueOffsetAdjuster { weight: 1.11 }),
            Rc::new(PercentReservationSlotAdjuster { percent: 4.6 }),
            Rc::new(FlaggedReservationSlotAdjuster {
                flags: flags.clone(),
                flag: "reservation-slot-fallback".to_string(),
                percent: -9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 76593.0,
    }
}
