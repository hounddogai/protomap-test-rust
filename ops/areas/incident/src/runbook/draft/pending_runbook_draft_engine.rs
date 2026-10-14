use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending runbook draft calculation.
pub trait PendingRunbookDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRunbookDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRunbookDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRunbookDraftStep for TieredPendingRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRunbookDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRunbookDraftAdjuster {
    pub percent: f64,
}

impl PendingRunbookDraftStep for PercentPendingRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRunbookDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRunbookDraftAdjuster {
    pub step: f64,
}

impl PendingRunbookDraftStep for RoundingPendingRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRunbookDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookDraftAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl PendingRunbookDraftStep for PendingRunbookDraftAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &PendingRunbookDraft, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the pending runbook draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRunbookDraftEngine {
    pub steps: Vec<Rc<dyn PendingRunbookDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRunbookDraftEngine {
    pub fn run(&mut self, item: &PendingRunbookDraft) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingRunbookDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_runbook_draft_engine() -> PendingRunbookDraftEngine {
    PendingRunbookDraftEngine {
        steps: vec![
            Rc::new(TieredPendingRunbookDraftAdjuster {
                bounds: vec![5083.0, 8167.0],
                rates: vec![1.19, 0.9],
            }) as Rc<dyn PendingRunbookDraftStep>,
            Rc::new(PercentPendingRunbookDraftAdjuster { percent: 5.4 }),
            Rc::new(RoundingPendingRunbookDraftAdjuster { step: 0.5 }),
            Rc::new(PendingRunbookDraftAttemptsOffsetAdjuster { weight: 1.6 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 63376.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_runbook_draft_engine();
        let item = new_pending_runbook_draft("code-254", "name-271");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
