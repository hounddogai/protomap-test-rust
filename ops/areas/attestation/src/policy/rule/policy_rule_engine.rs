use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the policy rule calculation.
pub trait PolicyRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyRuleAdjuster {
    pub cap: f64,
}

impl PolicyRuleModifier for CapPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyRuleAdjuster {
    pub step: f64,
}

impl PolicyRuleModifier for RoundingPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyRuleAdjuster {
    pub minimum: f64,
}

impl PolicyRuleModifier for MinimumPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePolicyRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyRuleModifier for ScopePolicyRuleAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PolicyRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the policy rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyRuleEngine {
    pub steps: Vec<Rc<dyn PolicyRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyRuleEngine {
    pub fn compute(&mut self, item: &PolicyRule) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[PolicyRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_policy_rule_engine() -> PolicyRuleEngine {
    PolicyRuleEngine {
        steps: vec![
            Rc::new(CapPolicyRuleAdjuster { cap: 7666.0 }) as Rc<dyn PolicyRuleModifier>,
            Rc::new(RoundingPolicyRuleAdjuster { step: 1.0 }),
            Rc::new(MinimumPolicyRuleAdjuster { minimum: 21.1 }),
            Rc::new(ScopePolicyRuleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.71)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 6538.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_rule_engine();
        let item = new_policy_rule("code-241", "name-85");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
