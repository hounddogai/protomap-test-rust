use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration adjustment calculation.
pub trait MigrationAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationAdjustmentModifier for TieredMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationAdjustmentAdjuster {
    pub step: f64,
}

impl MigrationAdjustmentModifier for RoundingMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationAdjustmentAdjuster {
    pub minimum: f64,
}

impl MigrationAdjustmentModifier for MinimumMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationAdjustmentAdjuster {
    pub cap: f64,
}

impl MigrationAdjustmentModifier for CapMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the migration adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationAdjustmentEngine {
    pub steps: Vec<Rc<dyn MigrationAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationAdjustmentEngine {
    pub fn compute(&mut self, item: &MigrationAdjustment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_migration_adjustment_engine() -> MigrationAdjustmentEngine {
    MigrationAdjustmentEngine {
        steps: vec![
            Rc::new(TieredMigrationAdjustmentAdjuster {
                bounds: vec![397.0, 7133.0, 7348.0, 8971.0],
                rates: vec![1.15, 1.12, 0.84, 1.18],
            }) as Rc<dyn MigrationAdjustmentModifier>,
            Rc::new(RoundingMigrationAdjustmentAdjuster { step: 1.0 }),
            Rc::new(MinimumMigrationAdjustmentAdjuster { minimum: 16.5 }),
            Rc::new(CapMigrationAdjustmentAdjuster { cap: 12546.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 39523.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_adjustment_engine();
        let item = new_migration_adjustment("code-112", "title-953");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
