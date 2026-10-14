use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive delegation override calculation.
pub trait AdaptiveDelegationOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveDelegationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveDelegationOverrideAdjuster {
    pub cap: f64,
}

impl AdaptiveDelegationOverrideModifier for CapAdaptiveDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveDelegationOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveDelegationOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveDelegationOverrideModifier for FlaggedAdaptiveDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveDelegationOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveDelegationOverrideAdjuster {
    pub percent: f64,
}

impl AdaptiveDelegationOverrideModifier for PercentAdaptiveDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveDelegationOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveDelegationOverrideAdjuster {
    pub step: f64,
}

impl AdaptiveDelegationOverrideModifier for RoundingAdaptiveDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveDelegationOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationOverrideGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveDelegationOverrideModifier for AdaptiveDelegationOverrideGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveDelegationOverride, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the adaptive delegation override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationOverrideEngine {
    pub steps: Vec<Rc<dyn AdaptiveDelegationOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveDelegationOverrideEngine {
    pub fn evaluate(&mut self, item: &AdaptiveDelegationOverride) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveDelegationOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveDelegationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_delegation_override_engine(flags: &flags::Registry) -> AdaptiveDelegationOverrideEngine {
    AdaptiveDelegationOverrideEngine {
        steps: vec![
            Rc::new(CapAdaptiveDelegationOverrideAdjuster { cap: 49124.0 })
                as Rc<dyn AdaptiveDelegationOverrideModifier>,
            Rc::new(FlaggedAdaptiveDelegationOverrideAdjuster {
                flags: flags.clone(),
                flag: "adaptive-delegation-override-primary".to_string(),
                percent: -2.1,
            }),
            Rc::new(PercentAdaptiveDelegationOverrideAdjuster { percent: 13.1 }),
            Rc::new(RoundingAdaptiveDelegationOverrideAdjuster { step: 1.0 }),
            Rc::new(AdaptiveDelegationOverrideGraceDaysOffsetAdjuster { weight: 0.29 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 48370.9,
    }
}
