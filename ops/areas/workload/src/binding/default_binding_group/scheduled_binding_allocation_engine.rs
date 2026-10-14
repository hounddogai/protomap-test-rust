use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled binding allocation calculation.
pub trait ScheduledBindingAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledBindingAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledBindingAllocationAdjuster {
    pub step: f64,
}

impl ScheduledBindingAllocationStep for RoundingScheduledBindingAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledBindingAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledBindingAllocationAdjuster {
    pub minimum: f64,
}

impl ScheduledBindingAllocationStep for MinimumScheduledBindingAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledBindingAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocationQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledBindingAllocationStep for ScheduledBindingAllocationQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledBindingAllocation, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the scheduled binding allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledBindingAllocationEngine {
    pub steps: Vec<Rc<dyn ScheduledBindingAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledBindingAllocationEngine {
    pub fn compute(&mut self, item: &ScheduledBindingAllocation) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledBindingAllocationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledBindingAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_binding_allocation_engine() -> ScheduledBindingAllocationEngine {
    ScheduledBindingAllocationEngine {
        steps: vec![
            Rc::new(RoundingScheduledBindingAllocationAdjuster { step: 5.0 }) as Rc<dyn ScheduledBindingAllocationStep>,
            Rc::new(MinimumScheduledBindingAllocationAdjuster { minimum: 21.3 }),
            Rc::new(ScheduledBindingAllocationQuotaUnitsOffsetAdjuster { weight: 0.48 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 71905.0,
    }
}
