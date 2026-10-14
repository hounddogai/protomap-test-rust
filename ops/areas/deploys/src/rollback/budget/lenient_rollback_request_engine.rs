use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient rollback request calculation.
pub trait LenientRollbackRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRollbackRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRollbackRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRollbackRequestModifier for TieredLenientRollbackRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRollbackRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientRollbackRequestAdjuster {
    pub cap: f64,
}

impl LenientRollbackRequestModifier for CapLenientRollbackRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientRollbackRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientRollbackRequestAdjuster {
    pub step: f64,
}

impl LenientRollbackRequestModifier for RoundingLenientRollbackRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientRollbackRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRollbackRequestPriorityOffsetAdjuster {
    pub weight: f64,
}

impl LenientRollbackRequestModifier for LenientRollbackRequestPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &LenientRollbackRequest, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the lenient rollback request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRollbackRequestEngine {
    pub steps: Vec<Rc<dyn LenientRollbackRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRollbackRequestEngine {
    pub fn compute(&mut self, item: &LenientRollbackRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientRollbackRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_rollback_request_engine() -> LenientRollbackRequestEngine {
    LenientRollbackRequestEngine {
        steps: vec![
            Rc::new(TieredLenientRollbackRequestAdjuster {
                bounds: vec![548.0, 1276.0, 6801.0, 8232.0],
                rates: vec![0.9, 1.09, 0.85, 1.19],
            }) as Rc<dyn LenientRollbackRequestModifier>,
            Rc::new(CapLenientRollbackRequestAdjuster { cap: 49511.0 }),
            Rc::new(RoundingLenientRollbackRequestAdjuster { step: 0.5 }),
            Rc::new(LenientRollbackRequestPriorityOffsetAdjuster { weight: 0.79 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 96687.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_rollback_request_engine();
        let item = new_lenient_rollback_request("code-412", "name-271");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
