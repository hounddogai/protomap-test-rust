use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary override calculation.
pub trait CanaryOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverrideScoreOffsetAdjuster {
    pub weight: f64,
}

impl CanaryOverrideStep for CanaryOverrideScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryOverride, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryOverrideAdjuster {
    pub percent: f64,
}

impl CanaryOverrideStep for PercentCanaryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryOverrideAdjuster {
    pub cap: f64,
}

impl CanaryOverrideStep for CapCanaryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryOverrideStep for FlaggedCanaryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryOverrideAdjuster {
    pub minimum: f64,
}

impl CanaryOverrideStep for MinimumCanaryOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the canary override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryOverrideEngine {
    pub steps: Vec<Rc<dyn CanaryOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryOverrideEngine {
    pub fn price(&mut self, item: &CanaryOverride) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[CanaryOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_override_engine(flags: &flags::Registry) -> CanaryOverrideEngine {
    CanaryOverrideEngine {
        steps: vec![
            Rc::new(CanaryOverrideScoreOffsetAdjuster { weight: 1.39 }) as Rc<dyn CanaryOverrideStep>,
            Rc::new(PercentCanaryOverrideAdjuster { percent: -0.4 }),
            Rc::new(CapCanaryOverrideAdjuster { cap: 14651.0 }),
            Rc::new(FlaggedCanaryOverrideAdjuster {
                flags: flags.clone(),
                flag: "canary-override-pending".to_string(),
                percent: -6.4,
            }),
            Rc::new(MinimumCanaryOverrideAdjuster { minimum: 33.9 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 58149.6,
    }
}
