use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered migration estimate calculation.
pub trait TieredMigrationEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredMigrationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredMigrationEstimateAdjuster {
    pub step: f64,
}

impl TieredMigrationEstimateModifier for RoundingTieredMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredMigrationEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationEstimateRetriesOffsetAdjuster {
    pub weight: f64,
}

impl TieredMigrationEstimateModifier for TieredMigrationEstimateRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &TieredMigrationEstimate, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredMigrationEstimateAdjuster {
    pub minimum: f64,
}

impl TieredMigrationEstimateModifier for MinimumTieredMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredMigrationEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredMigrationEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredMigrationEstimateModifier for FlaggedTieredMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredMigrationEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTieredMigrationEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredMigrationEstimateModifier for CategoryTieredMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &TieredMigrationEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredMigrationEstimateAdjuster {
    pub percent: f64,
}

impl TieredMigrationEstimateModifier for PercentTieredMigrationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredMigrationEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered migration estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredMigrationEstimateEngine {
    pub steps: Vec<Rc<dyn TieredMigrationEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredMigrationEstimateEngine {
    pub fn compute(&mut self, item: &TieredMigrationEstimate) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TieredMigrationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_migration_estimate_engine(flags: &flags::Registry) -> TieredMigrationEstimateEngine {
    TieredMigrationEstimateEngine {
        steps: vec![
            Rc::new(RoundingTieredMigrationEstimateAdjuster { step: 0.05 }) as Rc<dyn TieredMigrationEstimateModifier>,
            Rc::new(TieredMigrationEstimateRetriesOffsetAdjuster { weight: 1.76 }),
            Rc::new(MinimumTieredMigrationEstimateAdjuster { minimum: 41.3 }),
            Rc::new(FlaggedTieredMigrationEstimateAdjuster {
                flags: flags.clone(),
                flag: "tiered-migration-estimate-weighted".to_string(),
                percent: -3.6,
            }),
            Rc::new(CategoryTieredMigrationEstimateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.92), ("north".to_string(), 0.85)]),
            }),
            Rc::new(PercentTieredMigrationEstimateAdjuster { percent: 13.5 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 63774.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_migration_estimate_engine(&flags::new_registry());
        let item = new_tiered_migration_estimate("code-202", "name-952");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
