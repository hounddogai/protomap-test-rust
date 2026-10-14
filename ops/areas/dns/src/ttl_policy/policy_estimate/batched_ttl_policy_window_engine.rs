use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched ttl policy window calculation.
pub trait BatchedTtlPolicyWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTtlPolicyWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedTtlPolicyWindowAdjuster {
    pub step: f64,
}

impl BatchedTtlPolicyWindowModifier for RoundingBatchedTtlPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedTtlPolicyWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedTtlPolicyWindowAdjuster {
    pub cap: f64,
}

impl BatchedTtlPolicyWindowModifier for CapBatchedTtlPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedTtlPolicyWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyWindowThresholdOffsetAdjuster {
    pub weight: f64,
}

impl BatchedTtlPolicyWindowModifier for BatchedTtlPolicyWindowThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedTtlPolicyWindow, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedTtlPolicyWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedTtlPolicyWindowModifier for FlaggedBatchedTtlPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedTtlPolicyWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched ttl policy window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTtlPolicyWindowEngine {
    pub steps: Vec<Rc<dyn BatchedTtlPolicyWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTtlPolicyWindowEngine {
    pub fn price(&mut self, item: &BatchedTtlPolicyWindow) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTtlPolicyWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_ttl_policy_window_engine(flags: &flags::Registry) -> BatchedTtlPolicyWindowEngine {
    BatchedTtlPolicyWindowEngine {
        steps: vec![
            Rc::new(RoundingBatchedTtlPolicyWindowAdjuster { step: 0.5 }) as Rc<dyn BatchedTtlPolicyWindowModifier>,
            Rc::new(CapBatchedTtlPolicyWindowAdjuster { cap: 39937.0 }),
            Rc::new(BatchedTtlPolicyWindowThresholdOffsetAdjuster { weight: 0.43 }),
            Rc::new(FlaggedBatchedTtlPolicyWindowAdjuster {
                flags: flags.clone(),
                flag: "batched-ttl-policy-window-legacy".to_string(),
                percent: -2.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 44357.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_ttl_policy_window_engine(&flags::new_registry());
        let item = new_batched_ttl_policy_window("code-285", "name-897");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
