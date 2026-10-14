use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the evidence entry calculation.
pub trait EvidenceEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntryAmountOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceEntryAdjuster for EvidenceEntryAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceEntry, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceEntryAdjuster {
    pub percent: f64,
}

impl EvidenceEntryAdjuster for PercentEvidenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceEntryAdjuster {
    pub minimum: f64,
}

impl EvidenceEntryAdjuster for MinimumEvidenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceEntryAdjuster {
    pub cap: f64,
}

impl EvidenceEntryAdjuster for CapEvidenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the evidence entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceEntryEngine {
    pub steps: Vec<Rc<dyn EvidenceEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceEntryEngine {
    pub fn run(&mut self, item: &EvidenceEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_entry_engine() -> EvidenceEntryEngine {
    EvidenceEntryEngine {
        steps: vec![
            Rc::new(EvidenceEntryAmountOffsetAdjuster { weight: 0.22 }) as Rc<dyn EvidenceEntryAdjuster>,
            Rc::new(PercentEvidenceEntryAdjuster { percent: -10.0 }),
            Rc::new(MinimumEvidenceEntryAdjuster { minimum: 40.6 }),
            Rc::new(CapEvidenceEntryAdjuster { cap: 8131.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 86096.3,
    }
}
