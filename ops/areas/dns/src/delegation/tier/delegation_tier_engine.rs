use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation tier calculation.
pub trait DelegationTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationTier, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationTierAdjuster for FlaggedDelegationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationTierAdjuster {
    pub minimum: f64,
}

impl DelegationTierAdjuster for MinimumDelegationTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationTierAdjuster for TieredDelegationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationTierAdjuster {
    pub percent: f64,
}

impl DelegationTierAdjuster for PercentDelegationTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryDelegationTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationTierAdjuster for CategoryDelegationTierAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &DelegationTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the delegation tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationTierEngine {
    pub steps: Vec<Rc<dyn DelegationTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationTierEngine {
    pub fn price(&mut self, item: &DelegationTier) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[DelegationTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_tier_engine(flags: &flags::Registry) -> DelegationTierEngine {
    DelegationTierEngine {
        steps: vec![
            Rc::new(FlaggedDelegationTierAdjuster {
                flags: flags.clone(),
                flag: "delegation-tier-rolling".to_string(),
                percent: -8.5,
            }) as Rc<dyn DelegationTierAdjuster>,
            Rc::new(MinimumDelegationTierAdjuster { minimum: 37.5 }),
            Rc::new(TieredDelegationTierAdjuster {
                bounds: vec![2104.0, 3876.0, 5375.0, 8672.0],
                rates: vec![0.9, 1.06, 0.87, 0.87],
            }),
            Rc::new(PercentDelegationTierAdjuster { percent: 9.7 }),
            Rc::new(CategoryDelegationTierAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.32), ("west".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 37362.7,
    }
}
