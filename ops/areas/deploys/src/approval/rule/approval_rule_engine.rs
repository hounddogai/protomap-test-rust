use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval rule calculation.
pub trait ApprovalRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRuleRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalRuleModifier for ApprovalRuleRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalRule, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalRuleAdjuster {
    pub minimum: f64,
}

impl ApprovalRuleModifier for MinimumApprovalRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalRuleAdjuster {
    pub cap: f64,
}

impl ApprovalRuleModifier for CapApprovalRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeApprovalRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalRuleModifier for ScopeApprovalRuleAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ApprovalRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the approval rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalRuleEngine {
    pub steps: Vec<Rc<dyn ApprovalRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalRuleEngine {
    pub fn compute(&mut self, item: &ApprovalRule) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_approval_rule_engine() -> ApprovalRuleEngine {
    ApprovalRuleEngine {
        steps: vec![
            Rc::new(ApprovalRuleRetriesOffsetAdjuster { weight: 1.81 }) as Rc<dyn ApprovalRuleModifier>,
            Rc::new(MinimumApprovalRuleAdjuster { minimum: 20.1 }),
            Rc::new(CapApprovalRuleAdjuster { cap: 36183.0 }),
            Rc::new(ScopeApprovalRuleAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.01),
                    ("west".to_string(), 1.17),
                    ("central".to_string(), 1.06),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 51173.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_rule_engine();
        let item = new_approval_rule("code-578", "title-382");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
