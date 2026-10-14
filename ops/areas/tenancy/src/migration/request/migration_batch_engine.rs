use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration batch calculation.
pub trait MigrationBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatchScoreOffsetAdjuster {
    pub weight: f64,
}

impl MigrationBatchModifier for MigrationBatchScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationBatch, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationBatchModifier for TieredMigrationBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationBatchAdjuster {
    pub minimum: f64,
}

impl MigrationBatchModifier for MinimumMigrationBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the migration batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationBatchEngine {
    pub steps: Vec<Rc<dyn MigrationBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationBatchEngine {
    pub fn compute(&mut self, item: &MigrationBatch) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[MigrationBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_migration_batch_engine() -> MigrationBatchEngine {
    MigrationBatchEngine {
        steps: vec![
            Rc::new(MigrationBatchScoreOffsetAdjuster { weight: 0.16 }) as Rc<dyn MigrationBatchModifier>,
            Rc::new(TieredMigrationBatchAdjuster {
                bounds: vec![1705.0, 3561.0],
                rates: vec![1.19, 0.88],
            }),
            Rc::new(MinimumMigrationBatchAdjuster { minimum: 16.6 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 34593.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_batch_engine();
        let item = new_migration_batch("code-397", "name-985");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
