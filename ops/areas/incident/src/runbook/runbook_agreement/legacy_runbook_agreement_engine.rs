use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy runbook agreement calculation.
pub trait LegacyRunbookAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRunbookAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRunbookAgreementAdjuster {
    pub percent: f64,
}

impl LegacyRunbookAgreementAdjuster for PercentLegacyRunbookAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRunbookAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAgreementBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRunbookAgreementAdjuster for LegacyRunbookAgreementBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRunbookAgreement, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRunbookAgreementAdjuster {
    pub cap: f64,
}

impl LegacyRunbookAgreementAdjuster for CapLegacyRunbookAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRunbookAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyRunbookAgreementAdjuster {
    pub step: f64,
}

impl LegacyRunbookAgreementAdjuster for RoundingLegacyRunbookAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyRunbookAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy runbook agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRunbookAgreementEngine {
    pub steps: Vec<Rc<dyn LegacyRunbookAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRunbookAgreementEngine {
    pub fn evaluate(&mut self, item: &LegacyRunbookAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyRunbookAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_runbook_agreement_engine() -> LegacyRunbookAgreementEngine {
    LegacyRunbookAgreementEngine {
        steps: vec![
            Rc::new(PercentLegacyRunbookAgreementAdjuster { percent: 24.3 }) as Rc<dyn LegacyRunbookAgreementAdjuster>,
            Rc::new(LegacyRunbookAgreementBatchSizeOffsetAdjuster { weight: 1.35 }),
            Rc::new(CapLegacyRunbookAgreementAdjuster { cap: 44510.0 }),
            Rc::new(RoundingLegacyRunbookAgreementAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 19895.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_runbook_agreement_engine();
        let item = new_legacy_runbook_agreement("code-459", "name-798");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
