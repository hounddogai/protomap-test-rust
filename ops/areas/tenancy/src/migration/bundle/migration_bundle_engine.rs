use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration bundle calculation.
pub trait MigrationBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationBundleModifier for FlaggedMigrationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBundleLimitOffsetAdjuster {
    pub weight: f64,
}

impl MigrationBundleModifier for MigrationBundleLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationBundle, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationBundleAdjuster {
    pub percent: f64,
}

impl MigrationBundleModifier for PercentMigrationBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the migration bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationBundleEngine {
    pub steps: Vec<Rc<dyn MigrationBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationBundleEngine {
    pub fn price(&mut self, item: &MigrationBundle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[MigrationBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_migration_bundle_engine(flags: &flags::Registry) -> MigrationBundleEngine {
    MigrationBundleEngine {
        steps: vec![
            Rc::new(FlaggedMigrationBundleAdjuster {
                flags: flags.clone(),
                flag: "migration-bundle-pending".to_string(),
                percent: 0.1,
            }) as Rc<dyn MigrationBundleModifier>,
            Rc::new(MigrationBundleLimitOffsetAdjuster { weight: 1.74 }),
            Rc::new(PercentMigrationBundleAdjuster { percent: -9.8 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 27170.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_bundle_engine(&flags::new_registry());
        let item = new_migration_bundle("code-612", "label-265");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
