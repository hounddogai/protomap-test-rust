use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling isolation policy calculation.
pub trait RollingIsolationPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingIsolationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationPolicyLimitOffsetAdjuster {
    pub weight: f64,
}

impl RollingIsolationPolicyAdjuster for RollingIsolationPolicyLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &RollingIsolationPolicy, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingIsolationPolicyAdjuster {
    pub minimum: f64,
}

impl RollingIsolationPolicyAdjuster for MinimumRollingIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingIsolationPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingIsolationPolicyAdjuster {
    pub step: f64,
}

impl RollingIsolationPolicyAdjuster for RoundingRollingIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingIsolationPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingIsolationPolicyAdjuster {
    pub percent: f64,
}

impl RollingIsolationPolicyAdjuster for PercentRollingIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingIsolationPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingIsolationPolicyAdjuster {
    pub cap: f64,
}

impl RollingIsolationPolicyAdjuster for CapRollingIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingIsolationPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingIsolationPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingIsolationPolicyAdjuster for FlaggedRollingIsolationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingIsolationPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling isolation policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingIsolationPolicyEngine {
    pub steps: Vec<Rc<dyn RollingIsolationPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingIsolationPolicyEngine {
    pub fn compute(&mut self, item: &RollingIsolationPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingIsolationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_isolation_policy_engine(flags: &flags::Registry) -> RollingIsolationPolicyEngine {
    RollingIsolationPolicyEngine {
        steps: vec![
            Rc::new(RollingIsolationPolicyLimitOffsetAdjuster { weight: 1.12 })
                as Rc<dyn RollingIsolationPolicyAdjuster>,
            Rc::new(MinimumRollingIsolationPolicyAdjuster { minimum: 37.9 }),
            Rc::new(RoundingRollingIsolationPolicyAdjuster { step: 0.5 }),
            Rc::new(PercentRollingIsolationPolicyAdjuster { percent: -19.0 }),
            Rc::new(CapRollingIsolationPolicyAdjuster { cap: 23310.0 }),
            Rc::new(FlaggedRollingIsolationPolicyAdjuster {
                flags: flags.clone(),
                flag: "rolling-isolation-policy-global".to_string(),
                percent: 7.3,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 22588.6,
    }
}
