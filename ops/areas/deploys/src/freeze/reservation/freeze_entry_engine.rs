use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze entry calculation.
pub trait FreezeEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeEntryAdjuster {
    pub minimum: f64,
}

impl FreezeEntryModifier for MinimumFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeEntryModifier for FlaggedFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeEntryAdjuster {
    pub percent: f64,
}

impl FreezeEntryModifier for PercentFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntryCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl FreezeEntryModifier for FreezeEntryCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeEntry, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the freeze entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeEntryEngine {
    pub steps: Vec<Rc<dyn FreezeEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeEntryEngine {
    pub fn compute(&mut self, item: &FreezeEntry) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[FreezeEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_freeze_entry_engine(flags: &flags::Registry) -> FreezeEntryEngine {
    FreezeEntryEngine {
        steps: vec![
            Rc::new(MinimumFreezeEntryAdjuster { minimum: 20.0 }) as Rc<dyn FreezeEntryModifier>,
            Rc::new(FlaggedFreezeEntryAdjuster {
                flags: flags.clone(),
                flag: "freeze-entry-lenient".to_string(),
                percent: -8.4,
            }),
            Rc::new(PercentFreezeEntryAdjuster { percent: -14.6 }),
            Rc::new(FreezeEntryCreatedAtOffsetAdjuster { weight: 0.69 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 18055.2,
    }
}
