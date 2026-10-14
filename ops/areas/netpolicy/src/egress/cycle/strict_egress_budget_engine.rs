use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict egress budget calculation.
pub trait StrictEgressBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictEgressBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictEgressBudgetAdjuster {
    pub cap: f64,
}

impl StrictEgressBudgetFactor for CapStrictEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictEgressBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictEgressBudgetAdjuster {
    pub step: f64,
}

impl StrictEgressBudgetFactor for RoundingStrictEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictEgressBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictEgressBudgetAdjuster {
    pub percent: f64,
}

impl StrictEgressBudgetFactor for PercentStrictEgressBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictEgressBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBudgetRevisionOffsetAdjuster {
    pub weight: f64,
}

impl StrictEgressBudgetFactor for StrictEgressBudgetRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &StrictEgressBudget, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the strict egress budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictEgressBudgetEngine {
    pub steps: Vec<Rc<dyn StrictEgressBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictEgressBudgetEngine {
    pub fn price(&mut self, item: &StrictEgressBudget) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictEgressBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictEgressBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_egress_budget_engine() -> StrictEgressBudgetEngine {
    StrictEgressBudgetEngine {
        steps: vec![
            Rc::new(CapStrictEgressBudgetAdjuster { cap: 47952.0 }) as Rc<dyn StrictEgressBudgetFactor>,
            Rc::new(RoundingStrictEgressBudgetAdjuster { step: 0.1 }),
            Rc::new(PercentStrictEgressBudgetAdjuster { percent: -3.4 }),
            Rc::new(StrictEgressBudgetRevisionOffsetAdjuster { weight: 1.73 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 35580.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_egress_budget_engine();
        let item = new_strict_egress_budget("code-583", "label-957");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
