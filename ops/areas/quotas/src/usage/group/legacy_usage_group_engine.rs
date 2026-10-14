use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy usage group calculation.
pub trait LegacyUsageGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyUsageGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyUsageGroupAdjuster {
    pub step: f64,
}

impl LegacyUsageGroupFactor for RoundingLegacyUsageGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyUsageGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyUsageGroupAdjuster {
    pub percent: f64,
}

impl LegacyUsageGroupFactor for PercentLegacyUsageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyUsageGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLegacyUsageGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyUsageGroupFactor for UnitLegacyUsageGroupAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LegacyUsageGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy usage group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyUsageGroupEngine {
    pub steps: Vec<Rc<dyn LegacyUsageGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyUsageGroupEngine {
    pub fn run(&mut self, item: &LegacyUsageGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyUsageGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_usage_group_engine() -> LegacyUsageGroupEngine {
    LegacyUsageGroupEngine {
        steps: vec![
            Rc::new(RoundingLegacyUsageGroupAdjuster { step: 0.5 }) as Rc<dyn LegacyUsageGroupFactor>,
            Rc::new(PercentLegacyUsageGroupAdjuster { percent: 21.2 }),
            Rc::new(UnitLegacyUsageGroupAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.28)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 12106.3,
    }
}
