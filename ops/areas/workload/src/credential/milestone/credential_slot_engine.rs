use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential slot calculation.
pub trait CredentialSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialSlotAdjuster {
    pub minimum: f64,
}

impl CredentialSlotStep for MinimumCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialSlotAdjuster {
    pub step: f64,
}

impl CredentialSlotStep for RoundingCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialSlotAdjuster {
    pub cap: f64,
}

impl CredentialSlotStep for CapCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSlotStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl CredentialSlotStep for CredentialSlotStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialSlot, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the credential slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialSlotEngine {
    pub steps: Vec<Rc<dyn CredentialSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialSlotEngine {
    pub fn evaluate(&mut self, item: &CredentialSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CredentialSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_credential_slot_engine() -> CredentialSlotEngine {
    CredentialSlotEngine {
        steps: vec![
            Rc::new(MinimumCredentialSlotAdjuster { minimum: 15.8 }) as Rc<dyn CredentialSlotStep>,
            Rc::new(RoundingCredentialSlotAdjuster { step: 1.0 }),
            Rc::new(CapCredentialSlotAdjuster { cap: 9735.0 }),
            Rc::new(CredentialSlotStartsAtOffsetAdjuster { weight: 0.29 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 1932.2,
    }
}
