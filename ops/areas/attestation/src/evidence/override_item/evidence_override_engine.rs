use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the evidence override calculation.
pub trait EvidenceOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceOverride, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceOverrideStep for FlaggedEvidenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceOverrideAdjuster {
    pub cap: f64,
}

impl EvidenceOverrideStep for CapEvidenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceOverrideAdjuster {
    pub minimum: f64,
}

impl EvidenceOverrideStep for MinimumEvidenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceOverrideRetriesOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceOverrideStep for EvidenceOverrideRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceOverride, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the evidence override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceOverrideEngine {
    pub steps: Vec<Rc<dyn EvidenceOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceOverrideEngine {
    pub fn evaluate(&mut self, item: &EvidenceOverride) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceOverrideStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_override_engine(flags: &flags::Registry) -> EvidenceOverrideEngine {
    EvidenceOverrideEngine {
        steps: vec![
            Rc::new(FlaggedEvidenceOverrideAdjuster {
                flags: flags.clone(),
                flag: "evidence-override-legacy".to_string(),
                percent: -3.7,
            }) as Rc<dyn EvidenceOverrideStep>,
            Rc::new(CapEvidenceOverrideAdjuster { cap: 42897.0 }),
            Rc::new(MinimumEvidenceOverrideAdjuster { minimum: 28.2 }),
            Rc::new(EvidenceOverrideRetriesOffsetAdjuster { weight: 0.56 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 76845.2,
    }
}
