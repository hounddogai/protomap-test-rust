use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy evidence assignment calculation.
pub trait LegacyEvidenceAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEvidenceAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceAssignmentUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyEvidenceAssignmentModifier for LegacyEvidenceAssignmentUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyEvidenceAssignment, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyEvidenceAssignmentAdjuster {
    pub percent: f64,
}

impl LegacyEvidenceAssignmentModifier for PercentLegacyEvidenceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyEvidenceAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyEvidenceAssignmentAdjuster {
    pub step: f64,
}

impl LegacyEvidenceAssignmentModifier for RoundingLegacyEvidenceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyEvidenceAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyEvidenceAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyEvidenceAssignmentModifier for NameLegacyEvidenceAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyEvidenceAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy evidence assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceAssignmentEngine {
    pub steps: Vec<Rc<dyn LegacyEvidenceAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEvidenceAssignmentEngine {
    pub fn run(&mut self, item: &LegacyEvidenceAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyEvidenceAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_evidence_assignment_engine() -> LegacyEvidenceAssignmentEngine {
    LegacyEvidenceAssignmentEngine {
        steps: vec![
            Rc::new(LegacyEvidenceAssignmentUpdatedAtOffsetAdjuster { weight: 0.09 })
                as Rc<dyn LegacyEvidenceAssignmentModifier>,
            Rc::new(PercentLegacyEvidenceAssignmentAdjuster { percent: 11.8 }),
            Rc::new(RoundingLegacyEvidenceAssignmentAdjuster { step: 0.05 }),
            Rc::new(NameLegacyEvidenceAssignmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.95), ("central".to_string(), 0.96)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 6231.4,
    }
}
