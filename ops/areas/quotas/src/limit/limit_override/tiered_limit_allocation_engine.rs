use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered limit allocation calculation.
pub trait TieredLimitAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredLimitAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredLimitAllocationAdjuster {
    pub cap: f64,
}

impl TieredLimitAllocationFactor for CapTieredLimitAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredLimitAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredLimitAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredLimitAllocationFactor for FlaggedTieredLimitAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredLimitAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredLimitAllocationAdjuster {
    pub step: f64,
}

impl TieredLimitAllocationFactor for RoundingTieredLimitAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredLimitAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTieredLimitAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredLimitAllocationFactor for SourceTieredLimitAllocationAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TieredLimitAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitAllocationAmountOffsetAdjuster {
    pub weight: f64,
}

impl TieredLimitAllocationFactor for TieredLimitAllocationAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TieredLimitAllocation, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredLimitAllocationAdjuster {
    pub minimum: f64,
}

impl TieredLimitAllocationFactor for MinimumTieredLimitAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredLimitAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered limit allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredLimitAllocationEngine {
    pub steps: Vec<Rc<dyn TieredLimitAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredLimitAllocationEngine {
    pub fn price(&mut self, item: &TieredLimitAllocation) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredLimitAllocationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredLimitAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_limit_allocation_engine(flags: &flags::Registry) -> TieredLimitAllocationEngine {
    TieredLimitAllocationEngine {
        steps: vec![
            Rc::new(CapTieredLimitAllocationAdjuster { cap: 24093.0 }) as Rc<dyn TieredLimitAllocationFactor>,
            Rc::new(FlaggedTieredLimitAllocationAdjuster {
                flags: flags.clone(),
                flag: "tiered-limit-allocation-strict".to_string(),
                percent: -9.7,
            }),
            Rc::new(RoundingTieredLimitAllocationAdjuster { step: 0.5 }),
            Rc::new(SourceTieredLimitAllocationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.78)]),
            }),
            Rc::new(TieredLimitAllocationAmountOffsetAdjuster { weight: 0.07 }),
            Rc::new(MinimumTieredLimitAllocationAdjuster { minimum: 5.3 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 62306.2,
    }
}
