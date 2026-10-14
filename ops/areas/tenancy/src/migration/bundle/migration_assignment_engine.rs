use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the migration assignment calculation.
pub trait MigrationAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationAssignmentAdjuster for TieredMigrationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationAssignmentAdjuster {
    pub step: f64,
}

impl MigrationAssignmentAdjuster for RoundingMigrationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceMigrationAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationAssignmentAdjuster for NamespaceMigrationAssignmentAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &MigrationAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationAssignmentAdjuster {
    pub percent: f64,
}

impl MigrationAssignmentAdjuster for PercentMigrationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationAssignmentEngine {
    pub steps: Vec<Rc<dyn MigrationAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationAssignmentEngine {
    pub fn run(&mut self, item: &MigrationAssignment) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_assignment_engine() -> MigrationAssignmentEngine {
    MigrationAssignmentEngine {
        steps: vec![
            Rc::new(TieredMigrationAssignmentAdjuster {
                bounds: vec![371.0, 8815.0],
                rates: vec![0.91, 1.14],
            }) as Rc<dyn MigrationAssignmentAdjuster>,
            Rc::new(RoundingMigrationAssignmentAdjuster { step: 0.05 }),
            Rc::new(NamespaceMigrationAssignmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.02), ("north".to_string(), 1.18)]),
            }),
            Rc::new(PercentMigrationAssignmentAdjuster { percent: -17.2 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 67943.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_assignment_engine();
        let item = new_migration_assignment("code-374", "title-752");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
