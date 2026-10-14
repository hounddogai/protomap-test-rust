use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary ttl policy batch calculation.
pub trait PrimaryTtlPolicyBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTtlPolicyBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryTtlPolicyBatchAdjuster {
    pub minimum: f64,
}

impl PrimaryTtlPolicyBatchFactor for MinimumPrimaryTtlPolicyBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryTtlPolicyBatchAdjuster {
    pub cap: f64,
}

impl PrimaryTtlPolicyBatchFactor for CapPrimaryTtlPolicyBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryTtlPolicyBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryTtlPolicyBatchFactor for TieredPrimaryTtlPolicyBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryTtlPolicyBatchAdjuster {
    pub step: f64,
}

impl PrimaryTtlPolicyBatchFactor for RoundingPrimaryTtlPolicyBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary ttl policy batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyBatchEngine {
    pub steps: Vec<Rc<dyn PrimaryTtlPolicyBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTtlPolicyBatchEngine {
    pub fn price(&mut self, item: &PrimaryTtlPolicyBatch) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTtlPolicyBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_ttl_policy_batch_engine() -> PrimaryTtlPolicyBatchEngine {
    PrimaryTtlPolicyBatchEngine {
        steps: vec![
            Rc::new(MinimumPrimaryTtlPolicyBatchAdjuster { minimum: 44.5 }) as Rc<dyn PrimaryTtlPolicyBatchFactor>,
            Rc::new(CapPrimaryTtlPolicyBatchAdjuster { cap: 21117.0 }),
            Rc::new(TieredPrimaryTtlPolicyBatchAdjuster {
                bounds: vec![2981.0, 9590.0],
                rates: vec![1.07, 1.06],
            }),
            Rc::new(RoundingPrimaryTtlPolicyBatchAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 56227.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_ttl_policy_batch_engine();
        let item = new_primary_ttl_policy_batch("code-536", "title-276");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
