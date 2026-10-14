use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration tier calculation.
pub trait MigrationTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationTierFactor for TieredMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationTierAdjuster {
    pub step: f64,
}

impl MigrationTierFactor for RoundingMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameMigrationTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationTierFactor for NameMigrationTierAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &MigrationTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationTierFactor for FlaggedMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationTierAdjuster {
    pub minimum: f64,
}

impl MigrationTierFactor for MinimumMigrationTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the migration tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationTierEngine {
    pub steps: Vec<Rc<dyn MigrationTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationTierEngine {
    pub fn run(&mut self, item: &MigrationTier) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationTierFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_tier_engine(flags: &flags::Registry) -> MigrationTierEngine {
    MigrationTierEngine {
        steps: vec![
            Rc::new(TieredMigrationTierAdjuster {
                bounds: vec![197.0, 3521.0, 7453.0],
                rates: vec![1.15, 0.85, 0.87],
            }) as Rc<dyn MigrationTierFactor>,
            Rc::new(RoundingMigrationTierAdjuster { step: 0.25 }),
            Rc::new(NameMigrationTierAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.02),
                    ("north".to_string(), 0.8),
                    ("central".to_string(), 0.95),
                ]),
            }),
            Rc::new(FlaggedMigrationTierAdjuster {
                flags: flags.clone(),
                flag: "migration-tier-scheduled".to_string(),
                percent: 4.7,
            }),
            Rc::new(MinimumMigrationTierAdjuster { minimum: 8.7 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 81675.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_tier_engine(&flags::new_registry());
        let item = new_migration_tier("code-273", "name-455");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
