use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback egress policy calculation.
pub trait FallbackEgressPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackEgressPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackEgressPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackEgressPolicyAdjuster for FlaggedFallbackEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackEgressPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackEgressPolicyAdjuster {
    pub minimum: f64,
}

impl FallbackEgressPolicyAdjuster for MinimumFallbackEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackEgressPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackEgressPolicyAdjuster {
    pub percent: f64,
}

impl FallbackEgressPolicyAdjuster for PercentFallbackEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackEgressPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackEgressPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackEgressPolicyAdjuster for TieredFallbackEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackEgressPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPolicyVersionOffsetAdjuster {
    pub weight: f64,
}

impl FallbackEgressPolicyAdjuster for FallbackEgressPolicyVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackEgressPolicy, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleFallbackEgressPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackEgressPolicyAdjuster for LocaleFallbackEgressPolicyAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &FallbackEgressPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback egress policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackEgressPolicyEngine {
    pub steps: Vec<Rc<dyn FallbackEgressPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackEgressPolicyEngine {
    pub fn compute(&mut self, item: &FallbackEgressPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackEgressPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_egress_policy_engine(flags: &flags::Registry) -> FallbackEgressPolicyEngine {
    FallbackEgressPolicyEngine {
        steps: vec![
            Rc::new(FlaggedFallbackEgressPolicyAdjuster {
                flags: flags.clone(),
                flag: "fallback-egress-policy-weighted".to_string(),
                percent: -8.5,
            }) as Rc<dyn FallbackEgressPolicyAdjuster>,
            Rc::new(MinimumFallbackEgressPolicyAdjuster { minimum: 8.0 }),
            Rc::new(PercentFallbackEgressPolicyAdjuster { percent: -0.6 }),
            Rc::new(TieredFallbackEgressPolicyAdjuster {
                bounds: vec![72.0, 2041.0, 4733.0, 8034.0],
                rates: vec![1.0, 0.94, 1.13, 0.8],
            }),
            Rc::new(FallbackEgressPolicyVersionOffsetAdjuster { weight: 0.74 }),
            Rc::new(LocaleFallbackEgressPolicyAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.78), ("retail".to_string(), 0.9)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 75302.5,
    }
}
