use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling nonce baseline calculation.
pub trait RollingNonceBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingNonceBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingNonceBaselineAdjuster {
    pub minimum: f64,
}

impl RollingNonceBaselineAdjuster for MinimumRollingNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingNonceBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingNonceBaselineAdjuster {
    pub cap: f64,
}

impl RollingNonceBaselineAdjuster for CapRollingNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingNonceBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingNonceBaselineAdjuster {
    pub step: f64,
}

impl RollingNonceBaselineAdjuster for RoundingRollingNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingNonceBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingNonceBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingNonceBaselineAdjuster for FlaggedRollingNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingNonceBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRollingNonceBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingNonceBaselineAdjuster for ChannelRollingNonceBaselineAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RollingNonceBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingNonceBaselineAdjuster {
    pub percent: f64,
}

impl RollingNonceBaselineAdjuster for PercentRollingNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingNonceBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling nonce baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingNonceBaselineEngine {
    pub steps: Vec<Rc<dyn RollingNonceBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingNonceBaselineEngine {
    pub fn price(&mut self, item: &RollingNonceBaseline) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingNonceBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_nonce_baseline_engine(flags: &flags::Registry) -> RollingNonceBaselineEngine {
    RollingNonceBaselineEngine {
        steps: vec![
            Rc::new(MinimumRollingNonceBaselineAdjuster { minimum: 19.8 }) as Rc<dyn RollingNonceBaselineAdjuster>,
            Rc::new(CapRollingNonceBaselineAdjuster { cap: 49003.0 }),
            Rc::new(RoundingRollingNonceBaselineAdjuster { step: 0.1 }),
            Rc::new(FlaggedRollingNonceBaselineAdjuster {
                flags: flags.clone(),
                flag: "rolling-nonce-baseline-rolling".to_string(),
                percent: -8.2,
            }),
            Rc::new(ChannelRollingNonceBaselineAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.87),
                    ("west".to_string(), 1.19),
                    ("central".to_string(), 1.2),
                ]),
            }),
            Rc::new(PercentRollingNonceBaselineAdjuster { percent: 16.5 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 59985.0,
    }
}
