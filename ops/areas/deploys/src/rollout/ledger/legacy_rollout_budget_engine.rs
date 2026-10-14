use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy rollout budget calculation.
pub trait LegacyRolloutBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRolloutBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRolloutBudgetAdjuster {
    pub minimum: f64,
}

impl LegacyRolloutBudgetModifier for MinimumLegacyRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRolloutBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRolloutBudgetAdjuster {
    pub percent: f64,
}

impl LegacyRolloutBudgetModifier for PercentLegacyRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRolloutBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRolloutBudgetAdjuster {
    pub cap: f64,
}

impl LegacyRolloutBudgetModifier for CapLegacyRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRolloutBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyRolloutBudgetAdjuster {
    pub step: f64,
}

impl LegacyRolloutBudgetModifier for RoundingLegacyRolloutBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyRolloutBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy rollout budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRolloutBudgetEngine {
    pub steps: Vec<Rc<dyn LegacyRolloutBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRolloutBudgetEngine {
    pub fn price(&mut self, item: &LegacyRolloutBudget) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyRolloutBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRolloutBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_rollout_budget_engine() -> LegacyRolloutBudgetEngine {
    LegacyRolloutBudgetEngine {
        steps: vec![
            Rc::new(MinimumLegacyRolloutBudgetAdjuster { minimum: 45.8 }) as Rc<dyn LegacyRolloutBudgetModifier>,
            Rc::new(PercentLegacyRolloutBudgetAdjuster { percent: -0.0 }),
            Rc::new(CapLegacyRolloutBudgetAdjuster { cap: 24689.0 }),
            Rc::new(RoundingLegacyRolloutBudgetAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 45096.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_rollout_budget_engine();
        let item = new_legacy_rollout_budget("code-45", "name-419");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
