use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback overage target calculation.
pub trait FallbackOverageTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackOverageTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackOverageTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackOverageTargetFactor for FlaggedFallbackOverageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackOverageTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackOverageTargetAdjuster {
    pub percent: f64,
}

impl FallbackOverageTargetFactor for PercentFallbackOverageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackOverageTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackOverageTargetAdjuster {
    pub minimum: f64,
}

impl FallbackOverageTargetFactor for MinimumFallbackOverageTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackOverageTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionFallbackOverageTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackOverageTargetFactor for RegionFallbackOverageTargetAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &FallbackOverageTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTargetGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl FallbackOverageTargetFactor for FallbackOverageTargetGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackOverageTarget, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackOverageTargetAdjuster {
    pub cap: f64,
}

impl FallbackOverageTargetFactor for CapFallbackOverageTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackOverageTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback overage target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackOverageTargetEngine {
    pub steps: Vec<Rc<dyn FallbackOverageTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackOverageTargetEngine {
    pub fn evaluate(&mut self, item: &FallbackOverageTarget) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackOverageTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_overage_target_engine(flags: &flags::Registry) -> FallbackOverageTargetEngine {
    FallbackOverageTargetEngine {
        steps: vec![
            Rc::new(FlaggedFallbackOverageTargetAdjuster {
                flags: flags.clone(),
                flag: "fallback-overage-target-capped".to_string(),
                percent: -7.7,
            }) as Rc<dyn FallbackOverageTargetFactor>,
            Rc::new(PercentFallbackOverageTargetAdjuster { percent: -22.2 }),
            Rc::new(MinimumFallbackOverageTargetAdjuster { minimum: 47.2 }),
            Rc::new(RegionFallbackOverageTargetAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.91), ("south".to_string(), 1.19)]),
            }),
            Rc::new(FallbackOverageTargetGraceDaysOffsetAdjuster { weight: 0.75 }),
            Rc::new(CapFallbackOverageTargetAdjuster { cap: 28178.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 77614.0,
    }
}
