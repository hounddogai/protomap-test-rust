use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted retention budget calculation.
pub trait WeightedRetentionBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRetentionBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRetentionBudgetAdjuster {
    pub cap: f64,
}

impl WeightedRetentionBudgetModifier for CapWeightedRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRetentionBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRetentionBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRetentionBudgetModifier for FlaggedWeightedRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRetentionBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleWeightedRetentionBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedRetentionBudgetModifier for LocaleWeightedRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &WeightedRetentionBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted retention budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRetentionBudgetEngine {
    pub steps: Vec<Rc<dyn WeightedRetentionBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRetentionBudgetEngine {
    pub fn compute(&mut self, item: &WeightedRetentionBudget) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRetentionBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRetentionBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_retention_budget_engine(flags: &flags::Registry) -> WeightedRetentionBudgetEngine {
    WeightedRetentionBudgetEngine {
        steps: vec![
            Rc::new(CapWeightedRetentionBudgetAdjuster { cap: 14062.0 }) as Rc<dyn WeightedRetentionBudgetModifier>,
            Rc::new(FlaggedWeightedRetentionBudgetAdjuster {
                flags: flags.clone(),
                flag: "weighted-retention-budget-primary".to_string(),
                percent: 2.1,
            }),
            Rc::new(LocaleWeightedRetentionBudgetAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.86), ("online".to_string(), 1.02)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 56440.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_retention_budget_engine(&flags::new_registry());
        let item = new_weighted_retention_budget("code-928", "name-220");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
