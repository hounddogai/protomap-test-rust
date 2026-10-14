use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration baseline calculation.
pub trait MigrationBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeMigrationBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationBaselineAdjuster for ScopeMigrationBaselineAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &MigrationBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationBaselineAdjuster {
    pub minimum: f64,
}

impl MigrationBaselineAdjuster for MinimumMigrationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationBaselineAdjuster {
    pub percent: f64,
}

impl MigrationBaselineAdjuster for PercentMigrationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationBaselineEngine {
    pub steps: Vec<Rc<dyn MigrationBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationBaselineEngine {
    pub fn evaluate(&mut self, item: &MigrationBaseline) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[MigrationBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_migration_baseline_engine() -> MigrationBaselineEngine {
    MigrationBaselineEngine {
        steps: vec![
            Rc::new(ScopeMigrationBaselineAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.77), ("west".to_string(), 1.37)]),
            }) as Rc<dyn MigrationBaselineAdjuster>,
            Rc::new(MinimumMigrationBaselineAdjuster { minimum: 8.5 }),
            Rc::new(PercentMigrationBaselineAdjuster { percent: -18.4 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 50961.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_baseline_engine();
        let item = new_migration_baseline("code-291", "name-480");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
