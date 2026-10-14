use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient reservation allocation calculation.
pub trait LenientReservationAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientReservationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientReservationAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientReservationAllocationAdjuster for FlaggedLenientReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientReservationAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientReservationAllocationAdjuster {
    pub cap: f64,
}

impl LenientReservationAllocationAdjuster for CapLenientReservationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientReservationAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeLenientReservationAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientReservationAllocationAdjuster for ScopeLenientReservationAllocationAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &LenientReservationAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient reservation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientReservationAllocationEngine {
    pub steps: Vec<Rc<dyn LenientReservationAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientReservationAllocationEngine {
    pub fn price(&mut self, item: &LenientReservationAllocation) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LenientReservationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_reservation_allocation_engine(flags: &flags::Registry) -> LenientReservationAllocationEngine {
    LenientReservationAllocationEngine {
        steps: vec![
            Rc::new(FlaggedLenientReservationAllocationAdjuster {
                flags: flags.clone(),
                flag: "lenient-reservation-allocation-pending".to_string(),
                percent: 8.9,
            }) as Rc<dyn LenientReservationAllocationAdjuster>,
            Rc::new(CapLenientReservationAllocationAdjuster { cap: 39459.0 }),
            Rc::new(ScopeLenientReservationAllocationAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.29), ("south".to_string(), 1.08)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 96303.8,
    }
}
