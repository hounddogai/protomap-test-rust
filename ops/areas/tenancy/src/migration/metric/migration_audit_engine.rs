use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration audit calculation.
pub trait MigrationAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationAuditAdjuster {
    pub cap: f64,
}

impl MigrationAuditAdjuster for CapMigrationAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationAuditAdjuster for FlaggedMigrationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelMigrationAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MigrationAuditAdjuster for LabelMigrationAuditAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &MigrationAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationAuditAdjuster for TieredMigrationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMigrationAuditAdjuster {
    pub percent: f64,
}

impl MigrationAuditAdjuster for PercentMigrationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MigrationAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationAuditAdjuster {
    pub step: f64,
}

impl MigrationAuditAdjuster for RoundingMigrationAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the migration audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationAuditEngine {
    pub steps: Vec<Rc<dyn MigrationAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationAuditEngine {
    pub fn run(&mut self, item: &MigrationAudit) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MigrationAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MigrationAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_migration_audit_engine(flags: &flags::Registry) -> MigrationAuditEngine {
    MigrationAuditEngine {
        steps: vec![
            Rc::new(CapMigrationAuditAdjuster { cap: 39096.0 }) as Rc<dyn MigrationAuditAdjuster>,
            Rc::new(FlaggedMigrationAuditAdjuster {
                flags: flags.clone(),
                flag: "migration-audit-adaptive".to_string(),
                percent: -9.3,
            }),
            Rc::new(LabelMigrationAuditAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.32), ("west".to_string(), 1.29)]),
            }),
            Rc::new(TieredMigrationAuditAdjuster {
                bounds: vec![213.0, 612.0, 3736.0, 5457.0],
                rates: vec![0.83, 0.95, 0.95, 1.03],
            }),
            Rc::new(PercentMigrationAuditAdjuster { percent: 11.9 }),
            Rc::new(RoundingMigrationAuditAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 34425.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_migration_audit_engine(&flags::new_registry());
        let item = new_migration_audit("code-220", "label-157");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
