use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default canary allocation calculation.
pub trait DefaultCanaryAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultCanaryAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultCanaryAllocationAdjuster {
    pub cap: f64,
}

impl DefaultCanaryAllocationStep for CapDefaultCanaryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultCanaryAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultCanaryAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultCanaryAllocationStep for FlaggedDefaultCanaryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultCanaryAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCanaryAllocationMinimumOffsetAdjuster {
    pub weight: f64,
}

impl DefaultCanaryAllocationStep for DefaultCanaryAllocationMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultCanaryAllocation, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the default canary allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultCanaryAllocationEngine {
    pub steps: Vec<Rc<dyn DefaultCanaryAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultCanaryAllocationEngine {
    pub fn evaluate(&mut self, item: &DefaultCanaryAllocation) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultCanaryAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_canary_allocation_engine(flags: &flags::Registry) -> DefaultCanaryAllocationEngine {
    DefaultCanaryAllocationEngine {
        steps: vec![
            Rc::new(CapDefaultCanaryAllocationAdjuster { cap: 3378.0 }) as Rc<dyn DefaultCanaryAllocationStep>,
            Rc::new(FlaggedDefaultCanaryAllocationAdjuster {
                flags: flags.clone(),
                flag: "default-canary-allocation-tiered".to_string(),
                percent: 5.1,
            }),
            Rc::new(DefaultCanaryAllocationMinimumOffsetAdjuster { weight: 0.38 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 42505.1,
    }
}
