use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default federation allocation calculation.
pub trait DefaultFederationAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultFederationAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationCostOffsetAdjuster {
    pub weight: f64,
}

impl DefaultFederationAllocationFactor for DefaultFederationAllocationCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultFederationAllocation, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultFederationAllocationAdjuster {
    pub minimum: f64,
}

impl DefaultFederationAllocationFactor for MinimumDefaultFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultFederationAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultFederationAllocationAdjuster {
    pub cap: f64,
}

impl DefaultFederationAllocationFactor for CapDefaultFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultFederationAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultFederationAllocationAdjuster {
    pub step: f64,
}

impl DefaultFederationAllocationFactor for RoundingDefaultFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultFederationAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultFederationAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultFederationAllocationFactor for TieredDefaultFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultFederationAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultFederationAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultFederationAllocationFactor for FlaggedDefaultFederationAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultFederationAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default federation allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultFederationAllocationEngine {
    pub steps: Vec<Rc<dyn DefaultFederationAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultFederationAllocationEngine {
    pub fn price(&mut self, item: &DefaultFederationAllocation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultFederationAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_federation_allocation_engine(flags: &flags::Registry) -> DefaultFederationAllocationEngine {
    DefaultFederationAllocationEngine {
        steps: vec![
            Rc::new(DefaultFederationAllocationCostOffsetAdjuster { weight: 0.76 })
                as Rc<dyn DefaultFederationAllocationFactor>,
            Rc::new(MinimumDefaultFederationAllocationAdjuster { minimum: 8.2 }),
            Rc::new(CapDefaultFederationAllocationAdjuster { cap: 11627.0 }),
            Rc::new(RoundingDefaultFederationAllocationAdjuster { step: 0.05 }),
            Rc::new(TieredDefaultFederationAllocationAdjuster {
                bounds: vec![1337.0, 2350.0, 6370.0],
                rates: vec![0.99, 0.9, 1.11],
            }),
            Rc::new(FlaggedDefaultFederationAllocationAdjuster {
                flags: flags.clone(),
                flag: "default-federation-allocation-legacy".to_string(),
                percent: -4.0,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 63124.3,
    }
}
