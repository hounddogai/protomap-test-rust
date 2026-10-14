use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the budget burn preference calculation.
pub trait BudgetBurnPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceBudgetBurnPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnPreferenceFactor for NamespaceBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &BudgetBurnPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnPreferenceAdjuster {
    pub cap: f64,
}

impl BudgetBurnPreferenceFactor for CapBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnPreferenceAdjuster {
    pub minimum: f64,
}

impl BudgetBurnPreferenceFactor for MinimumBudgetBurnPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the budget burn preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnPreferenceEngine {
    pub steps: Vec<Rc<dyn BudgetBurnPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnPreferenceEngine {
    pub fn run(&mut self, item: &BudgetBurnPreference) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnPreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_budget_burn_preference_engine() -> BudgetBurnPreferenceEngine {
    BudgetBurnPreferenceEngine {
        steps: vec![
            Rc::new(NamespaceBudgetBurnPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.35),
                    ("north".to_string(), 1.35),
                    ("west".to_string(), 0.97),
                ]),
            }) as Rc<dyn BudgetBurnPreferenceFactor>,
            Rc::new(CapBudgetBurnPreferenceAdjuster { cap: 37793.0 }),
            Rc::new(MinimumBudgetBurnPreferenceAdjuster { minimum: 34.9 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 45917.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_preference_engine();
        let item = new_budget_burn_preference("code-104", "title-405");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
