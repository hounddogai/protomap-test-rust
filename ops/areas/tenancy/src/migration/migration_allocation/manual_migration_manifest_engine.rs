use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual migration manifest calculation.
pub trait ManualMigrationManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualMigrationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentManualMigrationManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualMigrationManifestAdjuster for SegmentManualMigrationManifestAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ManualMigrationManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualMigrationManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualMigrationManifestAdjuster for FlaggedManualMigrationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualMigrationManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationManifestWeightOffsetAdjuster {
    pub weight: f64,
}

impl ManualMigrationManifestAdjuster for ManualMigrationManifestWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &ManualMigrationManifest, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualMigrationManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualMigrationManifestAdjuster for TieredManualMigrationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualMigrationManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual migration manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualMigrationManifestEngine {
    pub steps: Vec<Rc<dyn ManualMigrationManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualMigrationManifestEngine {
    pub fn price(&mut self, item: &ManualMigrationManifest) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[ManualMigrationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_migration_manifest_engine(flags: &flags::Registry) -> ManualMigrationManifestEngine {
    ManualMigrationManifestEngine {
        steps: vec![
            Rc::new(SegmentManualMigrationManifestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.12), ("central".to_string(), 0.75)]),
            }) as Rc<dyn ManualMigrationManifestAdjuster>,
            Rc::new(FlaggedManualMigrationManifestAdjuster {
                flags: flags.clone(),
                flag: "manual-migration-manifest-pending".to_string(),
                percent: 6.3,
            }),
            Rc::new(ManualMigrationManifestWeightOffsetAdjuster { weight: 1.99 }),
            Rc::new(TieredManualMigrationManifestAdjuster {
                bounds: vec![3807.0, 6752.0, 8504.0, 9211.0],
                rates: vec![0.94, 1.19, 1.06, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 95678.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_migration_manifest_engine(&flags::new_registry());
        let item = new_manual_migration_manifest("code-29", "title-635");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
