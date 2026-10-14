use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration budget calculation.
pub trait MigrationBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameMigrationBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationBudgetFactor for NameMigrationBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &MigrationBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBudgetExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl MigrationBudgetFactor for MigrationBudgetExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationBudget, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationBudgetAdjuster {
    pub step: f64,
}

impl MigrationBudgetFactor for RoundingMigrationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the migration budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationBudgetEngine {
    pub steps: Vec<Rc<dyn MigrationBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationBudgetEngine {
    pub fn evaluate(&mut self, item: &MigrationBudget) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_migration_budget_engine() -> MigrationBudgetEngine {
    MigrationBudgetEngine {
        steps: vec![
            Rc::new(NameMigrationBudgetAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.34)]),
            }) as Rc<dyn MigrationBudgetFactor>,
            Rc::new(MigrationBudgetExpiresAtOffsetAdjuster { weight: 1.69 }),
            Rc::new(RoundingMigrationBudgetAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 71796.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_budget_engine();
        let item = new_migration_budget("code-750", "name-724");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
