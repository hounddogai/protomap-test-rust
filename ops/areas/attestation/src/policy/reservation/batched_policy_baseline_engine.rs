use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched policy baseline calculation.
pub trait BatchedPolicyBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPolicyBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBatchedPolicyBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedPolicyBaselineModifier for CategoryBatchedPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BatchedPolicyBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedPolicyBaselineAdjuster {
    pub step: f64,
}

impl BatchedPolicyBaselineModifier for RoundingBatchedPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedPolicyBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedPolicyBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedPolicyBaselineModifier for FlaggedBatchedPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedPolicyBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedPolicyBaselineAdjuster {
    pub cap: f64,
}

impl BatchedPolicyBaselineModifier for CapBatchedPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedPolicyBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the batched policy baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPolicyBaselineEngine {
    pub steps: Vec<Rc<dyn BatchedPolicyBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPolicyBaselineEngine {
    pub fn run(&mut self, item: &BatchedPolicyBaseline) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedPolicyBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_policy_baseline_engine(flags: &flags::Registry) -> BatchedPolicyBaselineEngine {
    BatchedPolicyBaselineEngine {
        steps: vec![
            Rc::new(CategoryBatchedPolicyBaselineAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.12), ("retail".to_string(), 1.09)]),
            }) as Rc<dyn BatchedPolicyBaselineModifier>,
            Rc::new(RoundingBatchedPolicyBaselineAdjuster { step: 0.25 }),
            Rc::new(FlaggedBatchedPolicyBaselineAdjuster {
                flags: flags.clone(),
                flag: "batched-policy-baseline-pending".to_string(),
                percent: 5.1,
            }),
            Rc::new(CapBatchedPolicyBaselineAdjuster { cap: 15967.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 55015.8,
    }
}
