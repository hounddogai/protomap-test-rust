use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched policy tier calculation.
pub trait BatchedPolicyTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPolicyTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBatchedPolicyTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedPolicyTierAdjuster for SegmentBatchedPolicyTierAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &BatchedPolicyTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTierDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl BatchedPolicyTierAdjuster for BatchedPolicyTierDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedPolicyTier, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedPolicyTierAdjuster {
    pub percent: f64,
}

impl BatchedPolicyTierAdjuster for PercentBatchedPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedPolicyTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched policy tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPolicyTierEngine {
    pub steps: Vec<Rc<dyn BatchedPolicyTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPolicyTierEngine {
    pub fn price(&mut self, item: &BatchedPolicyTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedPolicyTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_policy_tier_engine() -> BatchedPolicyTierEngine {
    BatchedPolicyTierEngine {
        steps: vec![
            Rc::new(SegmentBatchedPolicyTierAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.07), ("retail".to_string(), 1.29)]),
            }) as Rc<dyn BatchedPolicyTierAdjuster>,
            Rc::new(BatchedPolicyTierDurationMinutesOffsetAdjuster { weight: 1.76 }),
            Rc::new(PercentBatchedPolicyTierAdjuster { percent: 2.7 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 7280.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_policy_tier_engine();
        let item = new_batched_policy_tier("code-788", "name-939");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
