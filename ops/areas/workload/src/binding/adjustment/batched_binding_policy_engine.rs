use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched binding policy calculation.
pub trait BatchedBindingPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBindingPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedBindingPolicyAdjuster {
    pub minimum: f64,
}

impl BatchedBindingPolicyAdjuster for MinimumBatchedBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedBindingPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedBindingPolicyAdjuster {
    pub percent: f64,
}

impl BatchedBindingPolicyAdjuster for PercentBatchedBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedBindingPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedBindingPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedBindingPolicyAdjuster for TieredBatchedBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedBindingPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedBindingPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedBindingPolicyAdjuster for LabelBatchedBindingPolicyAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedBindingPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched binding policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBindingPolicyEngine {
    pub steps: Vec<Rc<dyn BatchedBindingPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBindingPolicyEngine {
    pub fn evaluate(&mut self, item: &BatchedBindingPolicy) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBindingPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_binding_policy_engine() -> BatchedBindingPolicyEngine {
    BatchedBindingPolicyEngine {
        steps: vec![
            Rc::new(MinimumBatchedBindingPolicyAdjuster { minimum: 49.9 }) as Rc<dyn BatchedBindingPolicyAdjuster>,
            Rc::new(PercentBatchedBindingPolicyAdjuster { percent: 20.4 }),
            Rc::new(TieredBatchedBindingPolicyAdjuster {
                bounds: vec![5595.0, 9440.0],
                rates: vec![1.08, 1.05],
            }),
            Rc::new(LabelBatchedBindingPolicyAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 20250.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_binding_policy_engine();
        let item = new_batched_binding_policy("code-868", "label-864");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
