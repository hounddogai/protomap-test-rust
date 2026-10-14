use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional migration tier calculation.
pub trait RegionalMigrationTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalMigrationTier, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalMigrationTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalMigrationTierFactor for FlaggedRegionalMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalMigrationTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalMigrationTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalMigrationTierFactor for TieredRegionalMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalMigrationTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierThresholdOffsetAdjuster {
    pub weight: f64,
}

impl RegionalMigrationTierFactor for RegionalMigrationTierThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalMigrationTier, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalMigrationTierAdjuster {
    pub cap: f64,
}

impl RegionalMigrationTierFactor for CapRegionalMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalMigrationTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRegionalMigrationTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalMigrationTierFactor for NamespaceRegionalMigrationTierAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RegionalMigrationTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalMigrationTierAdjuster {
    pub step: f64,
}

impl RegionalMigrationTierFactor for RoundingRegionalMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalMigrationTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional migration tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalMigrationTierEngine {
    pub steps: Vec<Rc<dyn RegionalMigrationTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalMigrationTierEngine {
    pub fn evaluate(&mut self, item: &RegionalMigrationTier) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalMigrationTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_migration_tier_engine(flags: &flags::Registry) -> RegionalMigrationTierEngine {
    RegionalMigrationTierEngine {
        steps: vec![
            Rc::new(FlaggedRegionalMigrationTierAdjuster {
                flags: flags.clone(),
                flag: "regional-migration-tier-manual".to_string(),
                percent: 3.9,
            }) as Rc<dyn RegionalMigrationTierFactor>,
            Rc::new(TieredRegionalMigrationTierAdjuster {
                bounds: vec![2247.0, 8097.0, 9233.0, 9716.0],
                rates: vec![1.02, 1.17, 0.99, 0.94],
            }),
            Rc::new(RegionalMigrationTierThresholdOffsetAdjuster { weight: 1.93 }),
            Rc::new(CapRegionalMigrationTierAdjuster { cap: 2257.0 }),
            Rc::new(NamespaceRegionalMigrationTierAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.77), ("south".to_string(), 1.02)]),
            }),
            Rc::new(RoundingRegionalMigrationTierAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 90524.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_migration_tier_engine(&flags::new_registry());
        let item = new_regional_migration_tier("code-412", "title-823");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
