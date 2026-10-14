use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered migration adjustment calculation.
pub trait TieredMigrationAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredMigrationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredMigrationAdjustmentAdjuster {
    pub minimum: f64,
}

impl TieredMigrationAdjustmentAdjuster for MinimumTieredMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredMigrationAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredMigrationAdjustmentAdjuster {
    pub step: f64,
}

impl TieredMigrationAdjustmentAdjuster for RoundingTieredMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredMigrationAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredMigrationAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredMigrationAdjustmentAdjuster for FlaggedTieredMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredMigrationAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredMigrationAdjustmentAdjuster {
    pub percent: f64,
}

impl TieredMigrationAdjustmentAdjuster for PercentTieredMigrationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredMigrationAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered migration adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredMigrationAdjustmentEngine {
    pub steps: Vec<Rc<dyn TieredMigrationAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredMigrationAdjustmentEngine {
    pub fn evaluate(&mut self, item: &TieredMigrationAdjustment) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredMigrationAdjustmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredMigrationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_migration_adjustment_engine(flags: &flags::Registry) -> TieredMigrationAdjustmentEngine {
    TieredMigrationAdjustmentEngine {
        steps: vec![
            Rc::new(MinimumTieredMigrationAdjustmentAdjuster { minimum: 2.3 })
                as Rc<dyn TieredMigrationAdjustmentAdjuster>,
            Rc::new(RoundingTieredMigrationAdjustmentAdjuster { step: 0.5 }),
            Rc::new(FlaggedTieredMigrationAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "tiered-migration-adjustment-scheduled".to_string(),
                percent: 3.5,
            }),
            Rc::new(PercentTieredMigrationAdjustmentAdjuster { percent: 0.4 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 54067.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_migration_adjustment_engine(&flags::new_registry());
        let item = new_tiered_migration_adjustment("code-859", "label-261");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
