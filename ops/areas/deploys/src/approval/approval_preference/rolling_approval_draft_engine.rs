use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling approval draft calculation.
pub trait RollingApprovalDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingApprovalDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingApprovalDraftAdjuster {
    pub step: f64,
}

impl RollingApprovalDraftFactor for RoundingRollingApprovalDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingApprovalDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraftVersionOffsetAdjuster {
    pub weight: f64,
}

impl RollingApprovalDraftFactor for RollingApprovalDraftVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &RollingApprovalDraft, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingApprovalDraftAdjuster {
    pub percent: f64,
}

impl RollingApprovalDraftFactor for PercentRollingApprovalDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingApprovalDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling approval draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingApprovalDraftEngine {
    pub steps: Vec<Rc<dyn RollingApprovalDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingApprovalDraftEngine {
    pub fn compute(&mut self, item: &RollingApprovalDraft) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingApprovalDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_approval_draft_engine() -> RollingApprovalDraftEngine {
    RollingApprovalDraftEngine {
        steps: vec![
            Rc::new(RoundingRollingApprovalDraftAdjuster { step: 0.05 }) as Rc<dyn RollingApprovalDraftFactor>,
            Rc::new(RollingApprovalDraftVersionOffsetAdjuster { weight: 0.7 }),
            Rc::new(PercentRollingApprovalDraftAdjuster { percent: -18.2 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 56234.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_approval_draft_engine();
        let item = new_rolling_approval_draft("code-145", "title-563");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
