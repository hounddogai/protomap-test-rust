use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity tier calculation.
pub trait IdentityTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityTierModifier for TieredIdentityTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityTierModifier for FlaggedIdentityTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityTierAdjuster {
    pub minimum: f64,
}

impl IdentityTierModifier for MinimumIdentityTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the identity tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityTierEngine {
    pub steps: Vec<Rc<dyn IdentityTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityTierEngine {
    pub fn compute(&mut self, item: &IdentityTier) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[IdentityTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_tier_engine(flags: &flags::Registry) -> IdentityTierEngine {
    IdentityTierEngine {
        steps: vec![
            Rc::new(TieredIdentityTierAdjuster {
                bounds: vec![1781.0, 9360.0],
                rates: vec![1.15, 1.03],
            }) as Rc<dyn IdentityTierModifier>,
            Rc::new(FlaggedIdentityTierAdjuster {
                flags: flags.clone(),
                flag: "identity-tier-batched".to_string(),
                percent: 1.6,
            }),
            Rc::new(MinimumIdentityTierAdjuster { minimum: 8.8 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 49517.9,
    }
}
