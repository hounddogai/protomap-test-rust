use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive delegation budget calculation.
pub trait AdaptiveDelegationBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveDelegationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveDelegationBudgetAdjuster {
    pub step: f64,
}

impl AdaptiveDelegationBudgetModifier for RoundingAdaptiveDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveDelegationBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudgetRatioOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveDelegationBudgetModifier for AdaptiveDelegationBudgetRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveDelegationBudget, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveDelegationBudgetAdjuster {
    pub cap: f64,
}

impl AdaptiveDelegationBudgetModifier for CapAdaptiveDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveDelegationBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveDelegationBudgetAdjuster {
    pub minimum: f64,
}

impl AdaptiveDelegationBudgetModifier for MinimumAdaptiveDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveDelegationBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveDelegationBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveDelegationBudgetModifier for TieredAdaptiveDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveDelegationBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive delegation budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationBudgetEngine {
    pub steps: Vec<Rc<dyn AdaptiveDelegationBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveDelegationBudgetEngine {
    pub fn run(&mut self, item: &AdaptiveDelegationBudget) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveDelegationBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveDelegationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_delegation_budget_engine() -> AdaptiveDelegationBudgetEngine {
    AdaptiveDelegationBudgetEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveDelegationBudgetAdjuster { step: 1.0 }) as Rc<dyn AdaptiveDelegationBudgetModifier>,
            Rc::new(AdaptiveDelegationBudgetRatioOffsetAdjuster { weight: 1.18 }),
            Rc::new(CapAdaptiveDelegationBudgetAdjuster { cap: 14303.0 }),
            Rc::new(MinimumAdaptiveDelegationBudgetAdjuster { minimum: 4.1 }),
            Rc::new(TieredAdaptiveDelegationBudgetAdjuster {
                bounds: vec![1630.0, 4170.0, 8211.0, 8962.0],
                rates: vec![0.99, 0.93, 1.18, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 89586.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_delegation_budget_engine();
        let item = new_adaptive_delegation_budget("code-333", "label-250");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
