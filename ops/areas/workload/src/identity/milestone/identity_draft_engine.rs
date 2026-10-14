use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the identity draft calculation.
pub trait IdentityDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityDraftAdjuster {
    pub minimum: f64,
}

impl IdentityDraftModifier for MinimumIdentityDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityDraftAdjuster {
    pub percent: f64,
}

impl IdentityDraftModifier for PercentIdentityDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityDraftModifier for FlaggedIdentityDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityDraftAdjuster {
    pub cap: f64,
}

impl IdentityDraftModifier for CapIdentityDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the identity draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityDraftEngine {
    pub steps: Vec<Rc<dyn IdentityDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityDraftEngine {
    pub fn price(&mut self, item: &IdentityDraft) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_draft_engine(flags: &flags::Registry) -> IdentityDraftEngine {
    IdentityDraftEngine {
        steps: vec![
            Rc::new(MinimumIdentityDraftAdjuster { minimum: 10.4 }) as Rc<dyn IdentityDraftModifier>,
            Rc::new(PercentIdentityDraftAdjuster { percent: 11.8 }),
            Rc::new(FlaggedIdentityDraftAdjuster {
                flags: flags.clone(),
                flag: "identity-draft-default".to_string(),
                percent: 3.5,
            }),
            Rc::new(CapIdentityDraftAdjuster { cap: 13287.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 1922.0,
    }
}
