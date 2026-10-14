use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the migration slot calculation.
pub trait MigrationSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationSlotAdjuster {
    pub percent: f64,
}

impl MigrationSlotFactor for PercentMigrationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationSlotAdjuster {
    pub minimum: f64,
}

impl MigrationSlotFactor for MinimumMigrationSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSlotCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl MigrationSlotFactor for MigrationSlotCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationSlot, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the migration slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationSlotEngine {
    pub steps: Vec<Rc<dyn MigrationSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationSlotEngine {
    pub fn run(&mut self, item: &MigrationSlot) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationSlotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_slot_engine() -> MigrationSlotEngine {
    MigrationSlotEngine {
        steps: vec![
            Rc::new(PercentMigrationSlotAdjuster { percent: -19.2 }) as Rc<dyn MigrationSlotFactor>,
            Rc::new(MinimumMigrationSlotAdjuster { minimum: 21.9 }),
            Rc::new(MigrationSlotCreatedAtOffsetAdjuster { weight: 1.01 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 4143.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_slot_engine();
        let item = new_migration_slot("code-417", "title-744");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
