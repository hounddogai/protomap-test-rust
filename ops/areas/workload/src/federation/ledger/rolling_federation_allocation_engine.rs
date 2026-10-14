use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling federation allocation calculation.
pub trait RollingFederationAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingFederationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationPortOffsetAdjuster {
    pub weight: f64,
}

impl RollingFederationAllocationAdjuster for RollingFederationAllocationPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &RollingFederationAllocation, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingFederationAllocationAdjuster {
    pub percent: f64,
}

impl RollingFederationAllocationAdjuster for PercentRollingFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingFederationAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingFederationAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingFederationAllocationAdjuster for TieredRollingFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingFederationAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling federation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingFederationAllocationEngine {
    pub steps: Vec<Rc<dyn RollingFederationAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingFederationAllocationEngine {
    pub fn run(&mut self, item: &RollingFederationAllocation) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingFederationAllocationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingFederationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_federation_allocation_engine() -> RollingFederationAllocationEngine {
    RollingFederationAllocationEngine {
        steps: vec![
            Rc::new(RollingFederationAllocationPortOffsetAdjuster { weight: 1.15 })
                as Rc<dyn RollingFederationAllocationAdjuster>,
            Rc::new(PercentRollingFederationAllocationAdjuster { percent: 4.0 }),
            Rc::new(TieredRollingFederationAllocationAdjuster {
                bounds: vec![608.0, 611.0, 1995.0, 7748.0],
                rates: vec![1.14, 1.19, 1.06, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 44835.7,
    }
}
