use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback egress override calculation.
pub trait FallbackEgressOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackEgressOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackEgressOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackEgressOverrideAdjuster for TieredFallbackEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackEgressOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFallbackEgressOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackEgressOverrideAdjuster for SourceFallbackEgressOverrideAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &FallbackEgressOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackEgressOverrideAdjuster {
    pub percent: f64,
}

impl FallbackEgressOverrideAdjuster for PercentFallbackEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackEgressOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackEgressOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackEgressOverrideAdjuster for FlaggedFallbackEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackEgressOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackEgressOverrideAdjuster {
    pub step: f64,
}

impl FallbackEgressOverrideAdjuster for RoundingFallbackEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackEgressOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackEgressOverrideAdjuster {
    pub cap: f64,
}

impl FallbackEgressOverrideAdjuster for CapFallbackEgressOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackEgressOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback egress override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackEgressOverrideEngine {
    pub steps: Vec<Rc<dyn FallbackEgressOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackEgressOverrideEngine {
    pub fn price(&mut self, item: &FallbackEgressOverride) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackEgressOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_egress_override_engine(flags: &flags::Registry) -> FallbackEgressOverrideEngine {
    FallbackEgressOverrideEngine {
        steps: vec![
            Rc::new(TieredFallbackEgressOverrideAdjuster {
                bounds: vec![3168.0, 8542.0],
                rates: vec![1.18, 0.9],
            }) as Rc<dyn FallbackEgressOverrideAdjuster>,
            Rc::new(SourceFallbackEgressOverrideAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.24), ("south".to_string(), 1.05)]),
            }),
            Rc::new(PercentFallbackEgressOverrideAdjuster { percent: 12.1 }),
            Rc::new(FlaggedFallbackEgressOverrideAdjuster {
                flags: flags.clone(),
                flag: "fallback-egress-override-adaptive".to_string(),
                percent: -8.5,
            }),
            Rc::new(RoundingFallbackEgressOverrideAdjuster { step: 5.0 }),
            Rc::new(CapFallbackEgressOverrideAdjuster { cap: 12809.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 23601.2,
    }
}
