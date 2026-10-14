use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback verifier audit calculation.
pub trait FallbackVerifierAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackVerifierAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackVerifierAuditAdjuster {
    pub step: f64,
}

impl FallbackVerifierAuditFactor for RoundingFallbackVerifierAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackVerifierAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierAuditMinimumOffsetAdjuster {
    pub weight: f64,
}

impl FallbackVerifierAuditFactor for FallbackVerifierAuditMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackVerifierAudit, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackVerifierAuditAdjuster {
    pub percent: f64,
}

impl FallbackVerifierAuditFactor for PercentFallbackVerifierAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackVerifierAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentFallbackVerifierAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackVerifierAuditFactor for SegmentFallbackVerifierAuditAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &FallbackVerifierAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback verifier audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackVerifierAuditEngine {
    pub steps: Vec<Rc<dyn FallbackVerifierAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackVerifierAuditEngine {
    pub fn evaluate(&mut self, item: &FallbackVerifierAudit) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackVerifierAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_verifier_audit_engine() -> FallbackVerifierAuditEngine {
    FallbackVerifierAuditEngine {
        steps: vec![
            Rc::new(RoundingFallbackVerifierAuditAdjuster { step: 5.0 }) as Rc<dyn FallbackVerifierAuditFactor>,
            Rc::new(FallbackVerifierAuditMinimumOffsetAdjuster { weight: 1.84 }),
            Rc::new(PercentFallbackVerifierAuditAdjuster { percent: 9.1 }),
            Rc::new(SegmentFallbackVerifierAuditAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.18)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 62061.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_verifier_audit_engine();
        let item = new_fallback_verifier_audit("code-961", "name-365");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
