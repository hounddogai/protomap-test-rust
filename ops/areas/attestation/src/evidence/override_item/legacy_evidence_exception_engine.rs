use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy evidence exception calculation.
pub trait LegacyEvidenceExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEvidenceException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLegacyEvidenceExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyEvidenceExceptionModifier for RegionLegacyEvidenceExceptionAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LegacyEvidenceException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyEvidenceExceptionAdjuster {
    pub minimum: f64,
}

impl LegacyEvidenceExceptionModifier for MinimumLegacyEvidenceExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyEvidenceException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceExceptionMarginOffsetAdjuster {
    pub weight: f64,
}

impl LegacyEvidenceExceptionModifier for LegacyEvidenceExceptionMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyEvidenceException, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the legacy evidence exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceExceptionEngine {
    pub steps: Vec<Rc<dyn LegacyEvidenceExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEvidenceExceptionEngine {
    pub fn evaluate(&mut self, item: &LegacyEvidenceException) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEvidenceException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_evidence_exception_engine() -> LegacyEvidenceExceptionEngine {
    LegacyEvidenceExceptionEngine {
        steps: vec![
            Rc::new(RegionLegacyEvidenceExceptionAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.35),
                    ("west".to_string(), 1.15),
                    ("east".to_string(), 0.92),
                ]),
            }) as Rc<dyn LegacyEvidenceExceptionModifier>,
            Rc::new(MinimumLegacyEvidenceExceptionAdjuster { minimum: 24.0 }),
            Rc::new(LegacyEvidenceExceptionMarginOffsetAdjuster { weight: 1.88 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 21703.8,
    }
}
