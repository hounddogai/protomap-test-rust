use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the credential bundle calculation.
pub trait CredentialBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialBundleSequenceOffsetAdjuster {
    pub weight: f64,
}

impl CredentialBundleModifier for CredentialBundleSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialBundle, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialBundleAdjuster {
    pub cap: f64,
}

impl CredentialBundleModifier for CapCredentialBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialBundleModifier for NameCredentialBundleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the credential bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialBundleEngine {
    pub steps: Vec<Rc<dyn CredentialBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialBundleEngine {
    pub fn evaluate(&mut self, item: &CredentialBundle) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CredentialBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_credential_bundle_engine() -> CredentialBundleEngine {
    CredentialBundleEngine {
        steps: vec![
            Rc::new(CredentialBundleSequenceOffsetAdjuster { weight: 0.48 }) as Rc<dyn CredentialBundleModifier>,
            Rc::new(CapCredentialBundleAdjuster { cap: 30299.0 }),
            Rc::new(NameCredentialBundleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.24)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 78517.7,
    }
}
