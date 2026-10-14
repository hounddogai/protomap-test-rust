use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched policy quota calculation.
pub trait BatchedPolicyQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPolicyQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedPolicyQuotaAdjuster {
    pub percent: f64,
}

impl BatchedPolicyQuotaFactor for PercentBatchedPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedPolicyQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPolicyQuotaAdjuster {
    pub minimum: f64,
}

impl BatchedPolicyQuotaFactor for MinimumBatchedPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPolicyQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedPolicyQuotaAdjuster {
    pub cap: f64,
}

impl BatchedPolicyQuotaFactor for CapBatchedPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedPolicyQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedPolicyQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedPolicyQuotaFactor for FlaggedBatchedPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedPolicyQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched policy quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPolicyQuotaEngine {
    pub steps: Vec<Rc<dyn BatchedPolicyQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPolicyQuotaEngine {
    pub fn run(&mut self, item: &BatchedPolicyQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedPolicyQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_policy_quota_engine(flags: &flags::Registry) -> BatchedPolicyQuotaEngine {
    BatchedPolicyQuotaEngine {
        steps: vec![
            Rc::new(PercentBatchedPolicyQuotaAdjuster { percent: -21.4 }) as Rc<dyn BatchedPolicyQuotaFactor>,
            Rc::new(MinimumBatchedPolicyQuotaAdjuster { minimum: 30.6 }),
            Rc::new(CapBatchedPolicyQuotaAdjuster { cap: 46698.0 }),
            Rc::new(FlaggedBatchedPolicyQuotaAdjuster {
                flags: flags.clone(),
                flag: "batched-policy-quota-seasonal".to_string(),
                percent: 5.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 72549.5,
    }
}
