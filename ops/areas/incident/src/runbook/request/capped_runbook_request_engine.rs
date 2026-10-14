use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped runbook request calculation.
pub trait CappedRunbookRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRunbookRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRunbookRequestAdjuster {
    pub minimum: f64,
}

impl CappedRunbookRequestStep for MinimumCappedRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRunbookRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedRunbookRequestAdjuster {
    pub cap: f64,
}

impl CappedRunbookRequestStep for CapCappedRunbookRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedRunbookRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookRequestEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl CappedRunbookRequestStep for CappedRunbookRequestEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &CappedRunbookRequest, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the capped runbook request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRunbookRequestEngine {
    pub steps: Vec<Rc<dyn CappedRunbookRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRunbookRequestEngine {
    pub fn compute(&mut self, item: &CappedRunbookRequest) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[CappedRunbookRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_runbook_request_engine() -> CappedRunbookRequestEngine {
    CappedRunbookRequestEngine {
        steps: vec![
            Rc::new(MinimumCappedRunbookRequestAdjuster { minimum: 49.3 }) as Rc<dyn CappedRunbookRequestStep>,
            Rc::new(CapCappedRunbookRequestAdjuster { cap: 24344.0 }),
            Rc::new(CappedRunbookRequestEndsAtOffsetAdjuster { weight: 1.61 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 52234.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_runbook_request_engine();
        let item = new_capped_runbook_request("code-147", "label-995");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
