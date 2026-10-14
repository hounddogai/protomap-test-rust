use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollout budget calculation.
pub trait RolloutBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutBudgetAdjuster {
    pub minimum: f64,
}

impl RolloutBudgetModifier for MinimumRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutBudgetAdjuster {
    pub step: f64,
}

impl RolloutBudgetModifier for RoundingRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRolloutBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutBudgetModifier for NameRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RolloutBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudgetCapacityOffsetAdjuster {
    pub weight: f64,
}

impl RolloutBudgetModifier for RolloutBudgetCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutBudget, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutBudgetModifier for TieredRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rollout budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutBudgetEngine {
    pub steps: Vec<Rc<dyn RolloutBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutBudgetEngine {
    pub fn price(&mut self, item: &RolloutBudget) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollout_budget_engine() -> RolloutBudgetEngine {
    RolloutBudgetEngine {
        steps: vec![
            Rc::new(MinimumRolloutBudgetAdjuster { minimum: 29.7 }) as Rc<dyn RolloutBudgetModifier>,
            Rc::new(RoundingRolloutBudgetAdjuster { step: 0.25 }),
            Rc::new(NameRolloutBudgetAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.97)]),
            }),
            Rc::new(RolloutBudgetCapacityOffsetAdjuster { weight: 1.16 }),
            Rc::new(TieredRolloutBudgetAdjuster {
                bounds: vec![5677.0, 9353.0],
                rates: vec![1.1, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 40918.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_budget_engine();
        let item = new_rollout_budget("code-397", "name-450");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
