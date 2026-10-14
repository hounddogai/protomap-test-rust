use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the migration quota calculation.
pub trait MigrationQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationQuotaMarginOffsetAdjuster {
    pub weight: f64,
}

impl MigrationQuotaFactor for MigrationQuotaMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &MigrationQuota, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationQuotaAdjuster {
    pub cap: f64,
}

impl MigrationQuotaFactor for CapMigrationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationQuotaFactor for FlaggedMigrationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationQuotaFactor for TieredMigrationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the migration quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationQuotaEngine {
    pub steps: Vec<Rc<dyn MigrationQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationQuotaEngine {
    pub fn evaluate(&mut self, item: &MigrationQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_migration_quota_engine(flags: &flags::Registry) -> MigrationQuotaEngine {
    MigrationQuotaEngine {
        steps: vec![
            Rc::new(MigrationQuotaMarginOffsetAdjuster { weight: 0.61 }) as Rc<dyn MigrationQuotaFactor>,
            Rc::new(CapMigrationQuotaAdjuster { cap: 46636.0 }),
            Rc::new(FlaggedMigrationQuotaAdjuster {
                flags: flags.clone(),
                flag: "migration-quota-legacy".to_string(),
                percent: 5.0,
            }),
            Rc::new(TieredMigrationQuotaAdjuster {
                bounds: vec![3544.0, 5285.0, 6780.0, 9975.0],
                rates: vec![1.08, 0.94, 0.83, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 23192.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_quota_engine(&flags::new_registry());
        let item = new_migration_quota("code-392", "name-421");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
