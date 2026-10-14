use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension allocation calculation.
pub trait SuspensionAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAllocationAmountOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionAllocationAdjuster for SuspensionAllocationAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionAllocation, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionAllocationAdjuster {
    pub percent: f64,
}

impl SuspensionAllocationAdjuster for PercentSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionAllocationAdjuster for TieredSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionAllocationAdjuster {
    pub step: f64,
}

impl SuspensionAllocationAdjuster for RoundingSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionAllocationAdjuster {
    pub minimum: f64,
}

impl SuspensionAllocationAdjuster for MinimumSuspensionAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the suspension allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionAllocationEngine {
    pub steps: Vec<Rc<dyn SuspensionAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionAllocationEngine {
    pub fn price(&mut self, item: &SuspensionAllocation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionAllocationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_suspension_allocation_engine() -> SuspensionAllocationEngine {
    SuspensionAllocationEngine {
        steps: vec![
            Rc::new(SuspensionAllocationAmountOffsetAdjuster { weight: 0.92 }) as Rc<dyn SuspensionAllocationAdjuster>,
            Rc::new(PercentSuspensionAllocationAdjuster { percent: 20.4 }),
            Rc::new(TieredSuspensionAllocationAdjuster {
                bounds: vec![1559.0, 4528.0, 9074.0],
                rates: vec![0.85, 0.81, 1.17],
            }),
            Rc::new(RoundingSuspensionAllocationAdjuster { step: 1.0 }),
            Rc::new(MinimumSuspensionAllocationAdjuster { minimum: 35.1 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 96782.0,
    }
}
