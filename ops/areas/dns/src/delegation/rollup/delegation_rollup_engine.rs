use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation rollup calculation.
pub trait DelegationRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRollupStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl DelegationRollupModifier for DelegationRollupStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationRollup, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationRollupModifier for TieredDelegationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationRollupAdjuster {
    pub step: f64,
}

impl DelegationRollupModifier for RoundingDelegationRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationRollupAdjuster {
    pub percent: f64,
}

impl DelegationRollupModifier for PercentDelegationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationRollupModifier for FlaggedDelegationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationRollupAdjuster {
    pub minimum: f64,
}

impl DelegationRollupModifier for MinimumDelegationRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the delegation rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationRollupEngine {
    pub steps: Vec<Rc<dyn DelegationRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationRollupEngine {
    pub fn compute(&mut self, item: &DelegationRollup) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[DelegationRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_rollup_engine(flags: &flags::Registry) -> DelegationRollupEngine {
    DelegationRollupEngine {
        steps: vec![
            Rc::new(DelegationRollupStartsAtOffsetAdjuster { weight: 1.54 }) as Rc<dyn DelegationRollupModifier>,
            Rc::new(TieredDelegationRollupAdjuster {
                bounds: vec![872.0, 3410.0, 6772.0],
                rates: vec![1.04, 0.84, 0.94],
            }),
            Rc::new(RoundingDelegationRollupAdjuster { step: 0.05 }),
            Rc::new(PercentDelegationRollupAdjuster { percent: 9.0 }),
            Rc::new(FlaggedDelegationRollupAdjuster {
                flags: flags.clone(),
                flag: "delegation-rollup-global".to_string(),
                percent: -6.6,
            }),
            Rc::new(MinimumDelegationRollupAdjuster { minimum: 19.7 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 57620.4,
    }
}
