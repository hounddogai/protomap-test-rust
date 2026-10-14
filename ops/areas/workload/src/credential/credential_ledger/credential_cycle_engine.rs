use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential cycle calculation.
pub trait CredentialCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialCycleAdjuster {
    pub minimum: f64,
}

impl CredentialCycleModifier for MinimumCredentialCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialCycleAdjuster {
    pub cap: f64,
}

impl CredentialCycleModifier for CapCredentialCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialCycleAdjuster {
    pub percent: f64,
}

impl CredentialCycleModifier for PercentCredentialCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialCycleAdjuster {
    pub step: f64,
}

impl CredentialCycleModifier for RoundingCredentialCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the credential cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialCycleEngine {
    pub steps: Vec<Rc<dyn CredentialCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialCycleEngine {
    pub fn evaluate(&mut self, item: &CredentialCycle) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[CredentialCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_credential_cycle_engine() -> CredentialCycleEngine {
    CredentialCycleEngine {
        steps: vec![
            Rc::new(MinimumCredentialCycleAdjuster { minimum: 33.4 }) as Rc<dyn CredentialCycleModifier>,
            Rc::new(CapCredentialCycleAdjuster { cap: 6435.0 }),
            Rc::new(PercentCredentialCycleAdjuster { percent: -9.6 }),
            Rc::new(RoundingCredentialCycleAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 86417.6,
    }
}
