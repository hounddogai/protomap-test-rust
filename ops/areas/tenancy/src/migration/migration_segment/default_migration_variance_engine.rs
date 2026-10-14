use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default migration variance calculation.
pub trait DefaultMigrationVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultMigrationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl DefaultMigrationVarianceModifier for DefaultMigrationVarianceQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultMigrationVariance, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDefaultMigrationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultMigrationVarianceModifier for SegmentDefaultMigrationVarianceAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &DefaultMigrationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultMigrationVarianceAdjuster {
    pub minimum: f64,
}

impl DefaultMigrationVarianceModifier for MinimumDefaultMigrationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultMigrationVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultMigrationVarianceAdjuster {
    pub percent: f64,
}

impl DefaultMigrationVarianceModifier for PercentDefaultMigrationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultMigrationVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default migration variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultMigrationVarianceEngine {
    pub steps: Vec<Rc<dyn DefaultMigrationVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultMigrationVarianceEngine {
    pub fn run(&mut self, item: &DefaultMigrationVariance) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultMigrationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_migration_variance_engine() -> DefaultMigrationVarianceEngine {
    DefaultMigrationVarianceEngine {
        steps: vec![
            Rc::new(DefaultMigrationVarianceQuotaUnitsOffsetAdjuster { weight: 1.69 })
                as Rc<dyn DefaultMigrationVarianceModifier>,
            Rc::new(SegmentDefaultMigrationVarianceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.15), ("west".to_string(), 0.99)]),
            }),
            Rc::new(MinimumDefaultMigrationVarianceAdjuster { minimum: 6.9 }),
            Rc::new(PercentDefaultMigrationVarianceAdjuster { percent: 19.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 44477.6,
    }
}
