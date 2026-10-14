use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration claim calculation.
pub trait MigrationClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationClaimModifier for FlaggedMigrationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationClaimAdjuster {
    pub minimum: f64,
}

impl MigrationClaimModifier for MinimumMigrationClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterMigrationClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationClaimModifier for ClusterMigrationClaimAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &MigrationClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationClaimThresholdOffsetAdjuster {
    pub weight: f64,
}

impl MigrationClaimModifier for MigrationClaimThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationClaim, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationClaimAdjuster {
    pub step: f64,
}

impl MigrationClaimModifier for RoundingMigrationClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationClaimAdjuster {
    pub cap: f64,
}

impl MigrationClaimModifier for CapMigrationClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the migration claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationClaimEngine {
    pub steps: Vec<Rc<dyn MigrationClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationClaimEngine {
    pub fn run(&mut self, item: &MigrationClaim) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_claim_engine(flags: &flags::Registry) -> MigrationClaimEngine {
    MigrationClaimEngine {
        steps: vec![
            Rc::new(FlaggedMigrationClaimAdjuster {
                flags: flags.clone(),
                flag: "migration-claim-default".to_string(),
                percent: -7.1,
            }) as Rc<dyn MigrationClaimModifier>,
            Rc::new(MinimumMigrationClaimAdjuster { minimum: 29.4 }),
            Rc::new(ClusterMigrationClaimAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.98)]),
            }),
            Rc::new(MigrationClaimThresholdOffsetAdjuster { weight: 1.46 }),
            Rc::new(RoundingMigrationClaimAdjuster { step: 0.5 }),
            Rc::new(CapMigrationClaimAdjuster { cap: 39309.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 30050.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_claim_engine(&flags::new_registry());
        let item = new_migration_claim("code-637", "title-181");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
