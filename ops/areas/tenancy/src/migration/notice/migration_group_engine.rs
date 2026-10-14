use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the migration group calculation.
pub trait MigrationGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationGroupAdjuster {
    pub cap: f64,
}

impl MigrationGroupStep for CapMigrationGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationGroupAdjuster {
    pub minimum: f64,
}

impl MigrationGroupStep for MinimumMigrationGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleMigrationGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationGroupStep for TitleMigrationGroupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &MigrationGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationGroupStep for TieredMigrationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationGroupRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl MigrationGroupStep for MigrationGroupRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationGroup, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the migration group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationGroupEngine {
    pub steps: Vec<Rc<dyn MigrationGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationGroupEngine {
    pub fn run(&mut self, item: &MigrationGroup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_group_engine() -> MigrationGroupEngine {
    MigrationGroupEngine {
        steps: vec![
            Rc::new(CapMigrationGroupAdjuster { cap: 15570.0 }) as Rc<dyn MigrationGroupStep>,
            Rc::new(MinimumMigrationGroupAdjuster { minimum: 14.8 }),
            Rc::new(TitleMigrationGroupAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.79)]),
            }),
            Rc::new(TieredMigrationGroupAdjuster {
                bounds: vec![403.0, 6979.0],
                rates: vec![1.19, 1.02],
            }),
            Rc::new(MigrationGroupRotationDaysOffsetAdjuster { weight: 0.12 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 60823.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_group_engine();
        let item = new_migration_group("code-475", "title-476");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
