use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy quota calculation.
pub trait PolicyQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyQuotaAdjuster {
    pub minimum: f64,
}

impl PolicyQuotaFactor for MinimumPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyQuotaAdjuster {
    pub percent: f64,
}

impl PolicyQuotaFactor for PercentPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyQuotaAdjuster {
    pub cap: f64,
}

impl PolicyQuotaFactor for CapPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyQuotaAdjuster {
    pub step: f64,
}

impl PolicyQuotaFactor for RoundingPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyQuotaFactor for TieredPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyQuotaFactor for FlaggedPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the policy quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyQuotaEngine {
    pub steps: Vec<Rc<dyn PolicyQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyQuotaEngine {
    pub fn run(&mut self, item: &PolicyQuota) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[PolicyQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_policy_quota_engine(flags: &flags::Registry) -> PolicyQuotaEngine {
    PolicyQuotaEngine {
        steps: vec![
            Rc::new(MinimumPolicyQuotaAdjuster { minimum: 34.5 }) as Rc<dyn PolicyQuotaFactor>,
            Rc::new(PercentPolicyQuotaAdjuster { percent: -14.0 }),
            Rc::new(CapPolicyQuotaAdjuster { cap: 3246.0 }),
            Rc::new(RoundingPolicyQuotaAdjuster { step: 0.05 }),
            Rc::new(TieredPolicyQuotaAdjuster {
                bounds: vec![2940.0, 8492.0],
                rates: vec![1.02, 0.86],
            }),
            Rc::new(FlaggedPolicyQuotaAdjuster {
                flags: flags.clone(),
                flag: "policy-quota-pending".to_string(),
                percent: 2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 7084.1,
    }
}
