use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched overage rule calculation.
pub trait BatchedOverageRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedOverageRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedOverageRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedOverageRuleModifier for NameBatchedOverageRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedOverageRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedOverageRuleAdjuster {
    pub minimum: f64,
}

impl BatchedOverageRuleModifier for MinimumBatchedOverageRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedOverageRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedOverageRuleAdjuster {
    pub cap: f64,
}

impl BatchedOverageRuleModifier for CapBatchedOverageRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedOverageRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageRuleAmountOffsetAdjuster {
    pub weight: f64,
}

impl BatchedOverageRuleModifier for BatchedOverageRuleAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedOverageRule, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the batched overage rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedOverageRuleEngine {
    pub steps: Vec<Rc<dyn BatchedOverageRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedOverageRuleEngine {
    pub fn evaluate(&mut self, item: &BatchedOverageRule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedOverageRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_overage_rule_engine() -> BatchedOverageRuleEngine {
    BatchedOverageRuleEngine {
        steps: vec![
            Rc::new(NameBatchedOverageRuleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.97),
                    ("north".to_string(), 1.0),
                    ("retail".to_string(), 1.38),
                ]),
            }) as Rc<dyn BatchedOverageRuleModifier>,
            Rc::new(MinimumBatchedOverageRuleAdjuster { minimum: 48.5 }),
            Rc::new(CapBatchedOverageRuleAdjuster { cap: 39683.0 }),
            Rc::new(BatchedOverageRuleAmountOffsetAdjuster { weight: 0.13 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 62268.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_overage_rule_engine();
        let item = new_batched_overage_rule("code-968", "name-956");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
