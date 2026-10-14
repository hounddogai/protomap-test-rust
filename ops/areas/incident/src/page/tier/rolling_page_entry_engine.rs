use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling page entry calculation.
pub trait RollingPageEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPageEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingPageEntryAdjuster {
    pub minimum: f64,
}

impl RollingPageEntryModifier for MinimumRollingPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingPageEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageEntryQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RollingPageEntryModifier for RollingPageEntryQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPageEntry, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPageEntryAdjuster {
    pub step: f64,
}

impl RollingPageEntryModifier for RoundingRollingPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPageEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPageEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPageEntryModifier for FlaggedRollingPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPageEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRollingPageEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPageEntryModifier for CategoryRollingPageEntryAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RollingPageEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPageEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPageEntryModifier for TieredRollingPageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPageEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling page entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPageEntryEngine {
    pub steps: Vec<Rc<dyn RollingPageEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPageEntryEngine {
    pub fn price(&mut self, item: &RollingPageEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingPageEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_page_entry_engine(flags: &flags::Registry) -> RollingPageEntryEngine {
    RollingPageEntryEngine {
        steps: vec![
            Rc::new(MinimumRollingPageEntryAdjuster { minimum: 6.4 }) as Rc<dyn RollingPageEntryModifier>,
            Rc::new(RollingPageEntryQuantityOffsetAdjuster { weight: 1.42 }),
            Rc::new(RoundingRollingPageEntryAdjuster { step: 0.1 }),
            Rc::new(FlaggedRollingPageEntryAdjuster {
                flags: flags.clone(),
                flag: "rolling-page-entry-rolling".to_string(),
                percent: 6.5,
            }),
            Rc::new(CategoryRollingPageEntryAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.29),
                    ("retail".to_string(), 1.36),
                    ("north".to_string(), 1.38),
                ]),
            }),
            Rc::new(TieredRollingPageEntryAdjuster {
                bounds: vec![2418.0, 5295.0, 5827.0, 8889.0],
                rates: vec![1.14, 1.11, 0.93, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 15577.8,
    }
}
