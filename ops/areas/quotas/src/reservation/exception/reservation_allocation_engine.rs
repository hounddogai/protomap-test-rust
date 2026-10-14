use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation allocation calculation.
pub trait ReservationAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationAllocationFactor for FlaggedReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationAllocationFactor for TieredReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocationAmountOffsetAdjuster {
    pub weight: f64,
}

impl ReservationAllocationFactor for ReservationAllocationAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationAllocation, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationAllocationAdjuster {
    pub cap: f64,
}

impl ReservationAllocationFactor for CapReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationAllocationAdjuster {
    pub percent: f64,
}

impl ReservationAllocationFactor for PercentReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the reservation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationAllocationEngine {
    pub steps: Vec<Rc<dyn ReservationAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationAllocationEngine {
    pub fn price(&mut self, item: &ReservationAllocation) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[ReservationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_reservation_allocation_engine(flags: &flags::Registry) -> ReservationAllocationEngine {
    ReservationAllocationEngine {
        steps: vec![
            Rc::new(FlaggedReservationAllocationAdjuster {
                flags: flags.clone(),
                flag: "reservation-allocation-batched".to_string(),
                percent: 5.3,
            }) as Rc<dyn ReservationAllocationFactor>,
            Rc::new(TieredReservationAllocationAdjuster {
                bounds: vec![5208.0, 8597.0, 8953.0, 9755.0],
                rates: vec![0.86, 0.85, 1.04, 0.92],
            }),
            Rc::new(ReservationAllocationAmountOffsetAdjuster { weight: 0.96 }),
            Rc::new(CapReservationAllocationAdjuster { cap: 20448.0 }),
            Rc::new(PercentReservationAllocationAdjuster { percent: 8.5 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 69152.5,
    }
}
