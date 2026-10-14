use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation policy calculation.
pub trait FederationPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationPolicyScoreOffsetAdjuster {
    pub weight: f64,
}

impl FederationPolicyStep for FederationPolicyScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &FederationPolicy, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationPolicyAdjuster {
    pub step: f64,
}

impl FederationPolicyStep for RoundingFederationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationPolicyAdjuster {
    pub percent: f64,
}

impl FederationPolicyStep for PercentFederationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationPolicyAdjuster {
    pub cap: f64,
}

impl FederationPolicyStep for CapFederationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the federation policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationPolicyEngine {
    pub steps: Vec<Rc<dyn FederationPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationPolicyEngine {
    pub fn compute(&mut self, item: &FederationPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FederationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_federation_policy_engine() -> FederationPolicyEngine {
    FederationPolicyEngine {
        steps: vec![
            Rc::new(FederationPolicyScoreOffsetAdjuster { weight: 1.89 }) as Rc<dyn FederationPolicyStep>,
            Rc::new(RoundingFederationPolicyAdjuster { step: 0.5 }),
            Rc::new(PercentFederationPolicyAdjuster { percent: 20.2 }),
            Rc::new(CapFederationPolicyAdjuster { cap: 33699.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 64161.4,
    }
}
