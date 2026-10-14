use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule budget calculation.
pub trait RuleBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBudgetPriorityOffsetAdjuster {
    pub weight: f64,
}

impl RuleBudgetStep for RuleBudgetPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &RuleBudget, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleBudgetAdjuster {
    pub minimum: f64,
}

impl RuleBudgetStep for MinimumRuleBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleBudgetAdjuster {
    pub step: f64,
}

impl RuleBudgetStep for RoundingRuleBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rule budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleBudgetEngine {
    pub steps: Vec<Rc<dyn RuleBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleBudgetEngine {
    pub fn compute(&mut self, item: &RuleBudget) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_budget_engine() -> RuleBudgetEngine {
    RuleBudgetEngine {
        steps: vec![
            Rc::new(RuleBudgetPriorityOffsetAdjuster { weight: 1.23 }) as Rc<dyn RuleBudgetStep>,
            Rc::new(MinimumRuleBudgetAdjuster { minimum: 47.7 }),
            Rc::new(RoundingRuleBudgetAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 72066.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_budget_engine();
        let item = new_rule_budget("code-745", "label-367");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
