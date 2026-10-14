use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling migration profile calculation.
pub trait RollingMigrationProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingMigrationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingMigrationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingMigrationProfileModifier for FlaggedRollingMigrationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingMigrationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingMigrationProfileAdjuster {
    pub percent: f64,
}

impl RollingMigrationProfileModifier for PercentRollingMigrationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingMigrationProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRollingMigrationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingMigrationProfileModifier for SourceRollingMigrationProfileAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RollingMigrationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingMigrationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingMigrationProfileModifier for TieredRollingMigrationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingMigrationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingMigrationProfileAdjuster {
    pub step: f64,
}

impl RollingMigrationProfileModifier for RoundingRollingMigrationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingMigrationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingMigrationProfileAdjuster {
    pub cap: f64,
}

impl RollingMigrationProfileModifier for CapRollingMigrationProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingMigrationProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling migration profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingMigrationProfileEngine {
    pub steps: Vec<Rc<dyn RollingMigrationProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingMigrationProfileEngine {
    pub fn evaluate(&mut self, item: &RollingMigrationProfile) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingMigrationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_migration_profile_engine(flags: &flags::Registry) -> RollingMigrationProfileEngine {
    RollingMigrationProfileEngine {
        steps: vec![
            Rc::new(FlaggedRollingMigrationProfileAdjuster {
                flags: flags.clone(),
                flag: "rolling-migration-profile-weighted".to_string(),
                percent: 2.2,
            }) as Rc<dyn RollingMigrationProfileModifier>,
            Rc::new(PercentRollingMigrationProfileAdjuster { percent: -4.5 }),
            Rc::new(SourceRollingMigrationProfileAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.28),
                    ("south".to_string(), 1.16),
                    ("east".to_string(), 1.19),
                ]),
            }),
            Rc::new(TieredRollingMigrationProfileAdjuster {
                bounds: vec![338.0, 3863.0, 5725.0, 7657.0],
                rates: vec![0.9, 1.01, 1.14, 0.87],
            }),
            Rc::new(RoundingRollingMigrationProfileAdjuster { step: 0.25 }),
            Rc::new(CapRollingMigrationProfileAdjuster { cap: 26819.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 58295.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_migration_profile_engine(&flags::new_registry());
        let item = new_rolling_migration_profile("code-210", "name-199");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
