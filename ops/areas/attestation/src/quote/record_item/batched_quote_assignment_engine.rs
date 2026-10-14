use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched quote assignment calculation.
pub trait BatchedQuoteAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedQuoteAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedQuoteAssignmentAdjuster {
    pub cap: f64,
}

impl BatchedQuoteAssignmentFactor for CapBatchedQuoteAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedQuoteAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedQuoteAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedQuoteAssignmentFactor for NameBatchedQuoteAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedQuoteAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteAssignmentBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl BatchedQuoteAssignmentFactor for BatchedQuoteAssignmentBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedQuoteAssignment, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

/// Runs the batched quote assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedQuoteAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedQuoteAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedQuoteAssignmentEngine {
    pub fn compute(&mut self, item: &BatchedQuoteAssignment) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedQuoteAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_quote_assignment_engine() -> BatchedQuoteAssignmentEngine {
    BatchedQuoteAssignmentEngine {
        steps: vec![
            Rc::new(CapBatchedQuoteAssignmentAdjuster { cap: 45398.0 }) as Rc<dyn BatchedQuoteAssignmentFactor>,
            Rc::new(NameBatchedQuoteAssignmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.95), ("west".to_string(), 1.03)]),
            }),
            Rc::new(BatchedQuoteAssignmentBatchSizeOffsetAdjuster { weight: 1.28 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 70845.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_quote_assignment_engine();
        let item = new_batched_quote_assignment("code-246", "name-157");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
