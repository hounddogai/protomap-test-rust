use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global reservation allocation calculation.
pub trait GlobalReservationAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReservationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalReservationAllocationAdjuster {
    pub step: f64,
}

impl GlobalReservationAllocationStep for RoundingGlobalReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalReservationAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalReservationAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalReservationAllocationStep for FlaggedGlobalReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalReservationAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationAllocationEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReservationAllocationStep for GlobalReservationAllocationEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReservationAllocation, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalReservationAllocationAdjuster {
    pub cap: f64,
}

impl GlobalReservationAllocationStep for CapGlobalReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalReservationAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global reservation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReservationAllocationEngine {
    pub steps: Vec<Rc<dyn GlobalReservationAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReservationAllocationEngine {
    pub fn evaluate(&mut self, item: &GlobalReservationAllocation) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReservationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_reservation_allocation_engine(flags: &flags::Registry) -> GlobalReservationAllocationEngine {
    GlobalReservationAllocationEngine {
        steps: vec![
            Rc::new(RoundingGlobalReservationAllocationAdjuster { step: 5.0 })
                as Rc<dyn GlobalReservationAllocationStep>,
            Rc::new(FlaggedGlobalReservationAllocationAdjuster {
                flags: flags.clone(),
                flag: "global-reservation-allocation-batched".to_string(),
                percent: 6.2,
            }),
            Rc::new(GlobalReservationAllocationEndsAtOffsetAdjuster { weight: 1.97 }),
            Rc::new(CapGlobalReservationAllocationAdjuster { cap: 26505.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 41509.8,
    }
}
