use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default limit tier calculation.
pub trait DefaultLimitTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultLimitTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultLimitTierAdjuster {
    pub step: f64,
}

impl DefaultLimitTierStep for RoundingDefaultLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultLimitTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultLimitTierAdjuster {
    pub minimum: f64,
}

impl DefaultLimitTierStep for MinimumDefaultLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultLimitTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultLimitTierAdjuster {
    pub cap: f64,
}

impl DefaultLimitTierStep for CapDefaultLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultLimitTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultLimitTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultLimitTierStep for TitleDefaultLimitTierAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultLimitTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitTierMinimumOffsetAdjuster {
    pub weight: f64,
}

impl DefaultLimitTierStep for DefaultLimitTierMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultLimitTier, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultLimitTierAdjuster {
    pub percent: f64,
}

impl DefaultLimitTierStep for PercentDefaultLimitTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultLimitTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default limit tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultLimitTierEngine {
    pub steps: Vec<Rc<dyn DefaultLimitTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultLimitTierEngine {
    pub fn price(&mut self, item: &DefaultLimitTier) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultLimitTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_limit_tier_engine() -> DefaultLimitTierEngine {
    DefaultLimitTierEngine {
        steps: vec![
            Rc::new(RoundingDefaultLimitTierAdjuster { step: 5.0 }) as Rc<dyn DefaultLimitTierStep>,
            Rc::new(MinimumDefaultLimitTierAdjuster { minimum: 26.4 }),
            Rc::new(CapDefaultLimitTierAdjuster { cap: 31686.0 }),
            Rc::new(TitleDefaultLimitTierAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.82)]),
            }),
            Rc::new(DefaultLimitTierMinimumOffsetAdjuster { weight: 1.76 }),
            Rc::new(PercentDefaultLimitTierAdjuster { percent: 3.3 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 29473.1,
    }
}
