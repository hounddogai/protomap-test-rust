use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the lenient migration budget calculation.
pub trait LenientMigrationBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientMigrationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientMigrationBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientMigrationBudgetStep for TieredLenientMigrationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientMigrationBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationBudgetWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl LenientMigrationBudgetStep for LenientMigrationBudgetWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &LenientMigrationBudget, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientMigrationBudgetAdjuster {
    pub minimum: f64,
}

impl LenientMigrationBudgetStep for MinimumLenientMigrationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientMigrationBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient migration budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientMigrationBudgetEngine {
    pub steps: Vec<Rc<dyn LenientMigrationBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientMigrationBudgetEngine {
    pub fn price(&mut self, item: &LenientMigrationBudget) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientMigrationBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientMigrationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_migration_budget_engine() -> LenientMigrationBudgetEngine {
    LenientMigrationBudgetEngine {
        steps: vec![
            Rc::new(TieredLenientMigrationBudgetAdjuster {
                bounds: vec![3330.0, 4916.0],
                rates: vec![0.82, 1.18],
            }) as Rc<dyn LenientMigrationBudgetStep>,
            Rc::new(LenientMigrationBudgetWindowDaysOffsetAdjuster { weight: 1.75 }),
            Rc::new(MinimumLenientMigrationBudgetAdjuster { minimum: 13.8 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 69126.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_migration_budget_engine();
        let item = new_lenient_migration_budget("code-119", "title-503");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
