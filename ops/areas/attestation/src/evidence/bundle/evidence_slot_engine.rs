use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence slot calculation.
pub trait EvidenceSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceSlotAdjuster {
    pub minimum: f64,
}

impl EvidenceSlotModifier for MinimumEvidenceSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceSlotAdjuster {
    pub cap: f64,
}

impl EvidenceSlotModifier for CapEvidenceSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceSlotAdjuster {
    pub step: f64,
}

impl EvidenceSlotModifier for RoundingEvidenceSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the evidence slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceSlotEngine {
    pub steps: Vec<Rc<dyn EvidenceSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceSlotEngine {
    pub fn evaluate(&mut self, item: &EvidenceSlot) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_slot_engine() -> EvidenceSlotEngine {
    EvidenceSlotEngine {
        steps: vec![
            Rc::new(MinimumEvidenceSlotAdjuster { minimum: 18.4 }) as Rc<dyn EvidenceSlotModifier>,
            Rc::new(CapEvidenceSlotAdjuster { cap: 1017.0 }),
            Rc::new(RoundingEvidenceSlotAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 30262.3,
    }
}
