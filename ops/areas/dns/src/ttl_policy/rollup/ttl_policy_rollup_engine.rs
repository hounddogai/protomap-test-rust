use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy rollup calculation.
pub trait TtlPolicyRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyRollupAdjuster {
    pub step: f64,
}

impl TtlPolicyRollupStep for RoundingTtlPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyRollupAdjuster {
    pub minimum: f64,
}

impl TtlPolicyRollupStep for MinimumTtlPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyRollupStep for FlaggedTtlPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyRollupAdjuster {
    pub percent: f64,
}

impl TtlPolicyRollupStep for PercentTtlPolicyRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyRollupEngine {
    pub steps: Vec<Rc<dyn TtlPolicyRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyRollupEngine {
    pub fn run(&mut self, item: &TtlPolicyRollup) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyRollupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_rollup_engine(flags: &flags::Registry) -> TtlPolicyRollupEngine {
    TtlPolicyRollupEngine {
        steps: vec![
            Rc::new(RoundingTtlPolicyRollupAdjuster { step: 0.1 }) as Rc<dyn TtlPolicyRollupStep>,
            Rc::new(MinimumTtlPolicyRollupAdjuster { minimum: 35.0 }),
            Rc::new(FlaggedTtlPolicyRollupAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-rollup-tiered".to_string(),
                percent: 4.3,
            }),
            Rc::new(PercentTtlPolicyRollupAdjuster { percent: -13.7 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 56207.9,
    }
}
