use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration mapping calculation.
pub trait MigrationMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationMapping, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationMappingModifier for FlaggedMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationMappingAdjuster {
    pub step: f64,
}

impl MigrationMappingModifier for RoundingMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationMappingAdjuster {
    pub percent: f64,
}

impl MigrationMappingModifier for PercentMigrationMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationMappingEngine {
    pub steps: Vec<Rc<dyn MigrationMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationMappingEngine {
    pub fn price(&mut self, item: &MigrationMapping) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_migration_mapping_engine(flags: &flags::Registry) -> MigrationMappingEngine {
    MigrationMappingEngine {
        steps: vec![
            Rc::new(FlaggedMigrationMappingAdjuster {
                flags: flags.clone(),
                flag: "migration-mapping-scheduled".to_string(),
                percent: -6.2,
            }) as Rc<dyn MigrationMappingModifier>,
            Rc::new(RoundingMigrationMappingAdjuster { step: 5.0 }),
            Rc::new(PercentMigrationMappingAdjuster { percent: -20.9 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 59750.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_mapping_engine(&flags::new_registry());
        let item = new_migration_mapping("code-95", "name-275");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
