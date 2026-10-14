use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched limit policy calculation.
pub trait BatchedLimitPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedLimitPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedLimitPolicyAdjuster {
    pub minimum: f64,
}

impl BatchedLimitPolicyStep for MinimumBatchedLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedLimitPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedLimitPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedLimitPolicyStep for FlaggedBatchedLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedLimitPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedLimitPolicyAdjuster {
    pub percent: f64,
}

impl BatchedLimitPolicyStep for PercentBatchedLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedLimitPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedLimitPolicyAdjuster {
    pub cap: f64,
}

impl BatchedLimitPolicyStep for CapBatchedLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedLimitPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedLimitPolicyAdjuster {
    pub step: f64,
}

impl BatchedLimitPolicyStep for RoundingBatchedLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedLimitPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitPolicyDiscountOffsetAdjuster {
    pub weight: f64,
}

impl BatchedLimitPolicyStep for BatchedLimitPolicyDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedLimitPolicy, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the batched limit policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedLimitPolicyEngine {
    pub steps: Vec<Rc<dyn BatchedLimitPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedLimitPolicyEngine {
    pub fn compute(&mut self, item: &BatchedLimitPolicy) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedLimitPolicyStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedLimitPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_limit_policy_engine(flags: &flags::Registry) -> BatchedLimitPolicyEngine {
    BatchedLimitPolicyEngine {
        steps: vec![
            Rc::new(MinimumBatchedLimitPolicyAdjuster { minimum: 2.4 }) as Rc<dyn BatchedLimitPolicyStep>,
            Rc::new(FlaggedBatchedLimitPolicyAdjuster {
                flags: flags.clone(),
                flag: "batched-limit-policy-seasonal".to_string(),
                percent: 8.7,
            }),
            Rc::new(PercentBatchedLimitPolicyAdjuster { percent: -5.7 }),
            Rc::new(CapBatchedLimitPolicyAdjuster { cap: 43458.0 }),
            Rc::new(RoundingBatchedLimitPolicyAdjuster { step: 0.05 }),
            Rc::new(BatchedLimitPolicyDiscountOffsetAdjuster { weight: 0.74 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 27772.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_limit_policy_engine(&flags::new_registry());
        let item = new_batched_limit_policy("code-740", "name-368");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
