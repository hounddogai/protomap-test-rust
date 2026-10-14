use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched policy allocation calculation.
pub trait BatchedPolicyAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPolicyAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedPolicyAllocationAdjuster {
    pub percent: f64,
}

impl BatchedPolicyAllocationStep for PercentBatchedPolicyAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedPolicyAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedPolicyAllocationAdjuster {
    pub cap: f64,
}

impl BatchedPolicyAllocationStep for CapBatchedPolicyAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedPolicyAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAllocationRevisionOffsetAdjuster {
    pub weight: f64,
}

impl BatchedPolicyAllocationStep for BatchedPolicyAllocationRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedPolicyAllocation, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the batched policy allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPolicyAllocationEngine {
    pub steps: Vec<Rc<dyn BatchedPolicyAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPolicyAllocationEngine {
    pub fn evaluate(&mut self, item: &BatchedPolicyAllocation) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedPolicyAllocationStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[BatchedPolicyAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_policy_allocation_engine() -> BatchedPolicyAllocationEngine {
    BatchedPolicyAllocationEngine {
        steps: vec![
            Rc::new(PercentBatchedPolicyAllocationAdjuster { percent: 24.8 }) as Rc<dyn BatchedPolicyAllocationStep>,
            Rc::new(CapBatchedPolicyAllocationAdjuster { cap: 21427.0 }),
            Rc::new(BatchedPolicyAllocationRevisionOffsetAdjuster { weight: 0.65 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 36992.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_policy_allocation_engine();
        let item = new_batched_policy_allocation("code-41", "label-361");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
