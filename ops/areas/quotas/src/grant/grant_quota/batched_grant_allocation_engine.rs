use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched grant allocation calculation.
pub trait BatchedGrantAllocationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedGrantAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedGrantAllocationAdjuster {
    pub percent: f64,
}

impl BatchedGrantAllocationModifier for PercentBatchedGrantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedGrantAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedGrantAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedGrantAllocationModifier for TieredBatchedGrantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedGrantAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedGrantAllocationAdjuster {
    pub cap: f64,
}

impl BatchedGrantAllocationModifier for CapBatchedGrantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedGrantAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantAllocationWeightOffsetAdjuster {
    pub weight: f64,
}

impl BatchedGrantAllocationModifier for BatchedGrantAllocationWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedGrantAllocation, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeBatchedGrantAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedGrantAllocationModifier for ScopeBatchedGrantAllocationAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &BatchedGrantAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedGrantAllocationAdjuster {
    pub minimum: f64,
}

impl BatchedGrantAllocationModifier for MinimumBatchedGrantAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedGrantAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched grant allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedGrantAllocationEngine {
    pub steps: Vec<Rc<dyn BatchedGrantAllocationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedGrantAllocationEngine {
    pub fn price(&mut self, item: &BatchedGrantAllocation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedGrantAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_grant_allocation_engine() -> BatchedGrantAllocationEngine {
    BatchedGrantAllocationEngine {
        steps: vec![
            Rc::new(PercentBatchedGrantAllocationAdjuster { percent: 13.5 }) as Rc<dyn BatchedGrantAllocationModifier>,
            Rc::new(TieredBatchedGrantAllocationAdjuster {
                bounds: vec![1739.0, 2156.0, 4349.0, 7913.0],
                rates: vec![1.13, 1.15, 1.17, 0.86],
            }),
            Rc::new(CapBatchedGrantAllocationAdjuster { cap: 35126.0 }),
            Rc::new(BatchedGrantAllocationWeightOffsetAdjuster { weight: 1.86 }),
            Rc::new(ScopeBatchedGrantAllocationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.79)]),
            }),
            Rc::new(MinimumBatchedGrantAllocationAdjuster { minimum: 28.2 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 33392.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_grant_allocation_engine();
        let item = new_batched_grant_allocation("code-530", "title-510");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
