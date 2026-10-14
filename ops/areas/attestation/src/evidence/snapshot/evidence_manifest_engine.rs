use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence manifest calculation.
pub trait EvidenceManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceManifestAdjuster {
    pub step: f64,
}

impl EvidenceManifestModifier for RoundingEvidenceManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceManifestRatioOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceManifestModifier for EvidenceManifestRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceManifest, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceManifestAdjuster {
    pub percent: f64,
}

impl EvidenceManifestModifier for PercentEvidenceManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the evidence manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceManifestEngine {
    pub steps: Vec<Rc<dyn EvidenceManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceManifestEngine {
    pub fn compute(&mut self, item: &EvidenceManifest) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_evidence_manifest_engine() -> EvidenceManifestEngine {
    EvidenceManifestEngine {
        steps: vec![
            Rc::new(RoundingEvidenceManifestAdjuster { step: 0.05 }) as Rc<dyn EvidenceManifestModifier>,
            Rc::new(EvidenceManifestRatioOffsetAdjuster { weight: 0.72 }),
            Rc::new(PercentEvidenceManifestAdjuster { percent: -23.1 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 61420.6,
    }
}
