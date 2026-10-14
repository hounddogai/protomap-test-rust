use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional canary allocation calculation.
pub trait RegionalCanaryAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalCanaryAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalCanaryAllocationAdjuster {
    pub step: f64,
}

impl RegionalCanaryAllocationAdjuster for RoundingRegionalCanaryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalCanaryAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRegionalCanaryAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalCanaryAllocationAdjuster for ChannelRegionalCanaryAllocationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RegionalCanaryAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalCanaryAllocationAdjuster {
    pub cap: f64,
}

impl RegionalCanaryAllocationAdjuster for CapRegionalCanaryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalCanaryAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalCanaryAllocationAdjuster {
    pub minimum: f64,
}

impl RegionalCanaryAllocationAdjuster for MinimumRegionalCanaryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalCanaryAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalCanaryAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalCanaryAllocationAdjuster for TieredRegionalCanaryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalCanaryAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAllocationBurstOffsetAdjuster {
    pub weight: f64,
}

impl RegionalCanaryAllocationAdjuster for RegionalCanaryAllocationBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalCanaryAllocation, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the regional canary allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalCanaryAllocationEngine {
    pub steps: Vec<Rc<dyn RegionalCanaryAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalCanaryAllocationEngine {
    pub fn evaluate(&mut self, item: &RegionalCanaryAllocation) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalCanaryAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_canary_allocation_engine() -> RegionalCanaryAllocationEngine {
    RegionalCanaryAllocationEngine {
        steps: vec![
            Rc::new(RoundingRegionalCanaryAllocationAdjuster { step: 0.05 })
                as Rc<dyn RegionalCanaryAllocationAdjuster>,
            Rc::new(ChannelRegionalCanaryAllocationAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.4),
                    ("north".to_string(), 0.78),
                    ("central".to_string(), 1.12),
                ]),
            }),
            Rc::new(CapRegionalCanaryAllocationAdjuster { cap: 35289.0 }),
            Rc::new(MinimumRegionalCanaryAllocationAdjuster { minimum: 45.6 }),
            Rc::new(TieredRegionalCanaryAllocationAdjuster {
                bounds: vec![7382.0, 7759.0, 9798.0],
                rates: vec![1.04, 0.83, 1.14],
            }),
            Rc::new(RegionalCanaryAllocationBurstOffsetAdjuster { weight: 1.24 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 1622.2,
    }
}
