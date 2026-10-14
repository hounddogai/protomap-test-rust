use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global release entry calculation.
pub trait GlobalReleaseEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReleaseEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalReleaseEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalReleaseEntryModifier for FlaggedGlobalReleaseEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalReleaseEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalReleaseEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalReleaseEntryModifier for TieredGlobalReleaseEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalReleaseEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReleaseEntryAdjuster {
    pub minimum: f64,
}

impl GlobalReleaseEntryModifier for MinimumGlobalReleaseEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReleaseEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalReleaseEntryAdjuster {
    pub percent: f64,
}

impl GlobalReleaseEntryModifier for PercentGlobalReleaseEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalReleaseEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global release entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReleaseEntryEngine {
    pub steps: Vec<Rc<dyn GlobalReleaseEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReleaseEntryEngine {
    pub fn price(&mut self, item: &GlobalReleaseEntry) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalReleaseEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReleaseEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_release_entry_engine(flags: &flags::Registry) -> GlobalReleaseEntryEngine {
    GlobalReleaseEntryEngine {
        steps: vec![
            Rc::new(FlaggedGlobalReleaseEntryAdjuster {
                flags: flags.clone(),
                flag: "global-release-entry-rolling".to_string(),
                percent: 6.2,
            }) as Rc<dyn GlobalReleaseEntryModifier>,
            Rc::new(TieredGlobalReleaseEntryAdjuster {
                bounds: vec![6748.0, 8659.0],
                rates: vec![0.86, 1.02],
            }),
            Rc::new(MinimumGlobalReleaseEntryAdjuster { minimum: 28.9 }),
            Rc::new(PercentGlobalReleaseEntryAdjuster { percent: 12.4 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 85843.0,
    }
}
