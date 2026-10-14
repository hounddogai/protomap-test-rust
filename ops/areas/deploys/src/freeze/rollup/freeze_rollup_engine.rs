use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the freeze rollup calculation.
pub trait FreezeRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeRollupFactor for TieredFreezeRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRollupAmountOffsetAdjuster {
    pub weight: f64,
}

impl FreezeRollupFactor for FreezeRollupAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeRollup, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeRollupFactor for FlaggedFreezeRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeRollupAdjuster {
    pub minimum: f64,
}

impl FreezeRollupFactor for MinimumFreezeRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionFreezeRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeRollupFactor for RegionFreezeRollupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &FreezeRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeRollupAdjuster {
    pub cap: f64,
}

impl FreezeRollupFactor for CapFreezeRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the freeze rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeRollupEngine {
    pub steps: Vec<Rc<dyn FreezeRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeRollupEngine {
    pub fn compute(&mut self, item: &FreezeRollup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_freeze_rollup_engine(flags: &flags::Registry) -> FreezeRollupEngine {
    FreezeRollupEngine {
        steps: vec![
            Rc::new(TieredFreezeRollupAdjuster {
                bounds: vec![111.0, 1599.0, 2339.0],
                rates: vec![1.2, 1.17, 0.98],
            }) as Rc<dyn FreezeRollupFactor>,
            Rc::new(FreezeRollupAmountOffsetAdjuster { weight: 0.17 }),
            Rc::new(FlaggedFreezeRollupAdjuster {
                flags: flags.clone(),
                flag: "freeze-rollup-pending".to_string(),
                percent: 4.7,
            }),
            Rc::new(MinimumFreezeRollupAdjuster { minimum: 27.3 }),
            Rc::new(RegionFreezeRollupAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.14),
                    ("north".to_string(), 0.95),
                    ("west".to_string(), 1.35),
                ]),
            }),
            Rc::new(CapFreezeRollupAdjuster { cap: 29003.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 86080.0,
    }
}
