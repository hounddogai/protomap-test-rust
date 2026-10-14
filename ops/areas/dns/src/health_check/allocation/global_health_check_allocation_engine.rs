use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global health check allocation calculation.
pub trait GlobalHealthCheckAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalHealthCheckAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalHealthCheckAllocationAdjuster {
    pub cap: f64,
}

impl GlobalHealthCheckAllocationAdjuster for CapGlobalHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalHealthCheckAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleGlobalHealthCheckAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalHealthCheckAllocationAdjuster for TitleGlobalHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &GlobalHealthCheckAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalHealthCheckAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalHealthCheckAllocationAdjuster for TieredGlobalHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalHealthCheckAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalHealthCheckAllocationAdjuster {
    pub step: f64,
}

impl GlobalHealthCheckAllocationAdjuster for RoundingGlobalHealthCheckAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalHealthCheckAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global health check allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalHealthCheckAllocationEngine {
    pub steps: Vec<Rc<dyn GlobalHealthCheckAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalHealthCheckAllocationEngine {
    pub fn evaluate(&mut self, item: &GlobalHealthCheckAllocation) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalHealthCheckAllocationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalHealthCheckAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_health_check_allocation_engine() -> GlobalHealthCheckAllocationEngine {
    GlobalHealthCheckAllocationEngine {
        steps: vec![
            Rc::new(CapGlobalHealthCheckAllocationAdjuster { cap: 48623.0 })
                as Rc<dyn GlobalHealthCheckAllocationAdjuster>,
            Rc::new(TitleGlobalHealthCheckAllocationAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.11),
                    ("online".to_string(), 0.9),
                    ("north".to_string(), 1.3),
                ]),
            }),
            Rc::new(TieredGlobalHealthCheckAllocationAdjuster {
                bounds: vec![396.0, 1737.0, 5342.0, 8581.0],
                rates: vec![0.82, 1.17, 0.83, 0.97],
            }),
            Rc::new(RoundingGlobalHealthCheckAllocationAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 49583.9,
    }
}
