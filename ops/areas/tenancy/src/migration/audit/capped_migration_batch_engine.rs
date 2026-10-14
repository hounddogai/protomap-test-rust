use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped migration batch calculation.
pub trait CappedMigrationBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedMigrationBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedMigrationBatchAdjuster {
    pub cap: f64,
}

impl CappedMigrationBatchModifier for CapCappedMigrationBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedMigrationBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedMigrationBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedMigrationBatchModifier for TieredCappedMigrationBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedMigrationBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedMigrationBatchAdjuster {
    pub minimum: f64,
}

impl CappedMigrationBatchModifier for MinimumCappedMigrationBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedMigrationBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedMigrationBatchAdjuster {
    pub percent: f64,
}

impl CappedMigrationBatchModifier for PercentCappedMigrationBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedMigrationBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped migration batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedMigrationBatchEngine {
    pub steps: Vec<Rc<dyn CappedMigrationBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedMigrationBatchEngine {
    pub fn price(&mut self, item: &CappedMigrationBatch) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedMigrationBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_migration_batch_engine() -> CappedMigrationBatchEngine {
    CappedMigrationBatchEngine {
        steps: vec![
            Rc::new(CapCappedMigrationBatchAdjuster { cap: 35558.0 }) as Rc<dyn CappedMigrationBatchModifier>,
            Rc::new(TieredCappedMigrationBatchAdjuster {
                bounds: vec![4965.0, 5956.0, 9249.0],
                rates: vec![1.06, 0.82, 0.97],
            }),
            Rc::new(MinimumCappedMigrationBatchAdjuster { minimum: 21.4 }),
            Rc::new(PercentCappedMigrationBatchAdjuster { percent: -17.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 79616.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_migration_batch_engine();
        let item = new_capped_migration_batch("code-319", "name-570");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
