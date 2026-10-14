use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the postmortem allocation calculation.
pub trait PostmortemAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemAllocationAdjuster {
    pub step: f64,
}

impl PostmortemAllocationModifier for RoundingPostmortemAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemAllocationAdjuster {
    pub cap: f64,
}

impl PostmortemAllocationModifier for CapPostmortemAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemAllocationAdjuster {
    pub percent: f64,
}

impl PostmortemAllocationModifier for PercentPostmortemAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the postmortem allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemAllocationEngine {
    pub steps: Vec<Rc<dyn PostmortemAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemAllocationEngine {
    pub fn price(&mut self, item: &PostmortemAllocation) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_allocation_engine() -> PostmortemAllocationEngine {
    PostmortemAllocationEngine {
        steps: vec![
            Rc::new(RoundingPostmortemAllocationAdjuster { step: 5.0 }) as Rc<dyn PostmortemAllocationModifier>,
            Rc::new(CapPostmortemAllocationAdjuster { cap: 43018.0 }),
            Rc::new(PercentPostmortemAllocationAdjuster { percent: -19.1 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 76463.8,
    }
}
