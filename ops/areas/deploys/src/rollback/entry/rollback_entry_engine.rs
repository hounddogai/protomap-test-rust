use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rollback entry calculation.
pub trait RollbackEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackEntryModifier for TieredRollbackEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackEntryAdjuster {
    pub percent: f64,
}

impl RollbackEntryModifier for PercentRollbackEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollbackEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackEntryModifier for NameRollbackEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollbackEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackEntryEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl RollbackEntryModifier for RollbackEntryEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackEntry, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackEntryModifier for FlaggedRollbackEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackEntryEngine {
    pub steps: Vec<Rc<dyn RollbackEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackEntryEngine {
    pub fn price(&mut self, item: &RollbackEntry) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_entry_engine(flags: &flags::Registry) -> RollbackEntryEngine {
    RollbackEntryEngine {
        steps: vec![
            Rc::new(TieredRollbackEntryAdjuster {
                bounds: vec![4464.0, 6884.0, 8160.0],
                rates: vec![0.9, 0.96, 0.89],
            }) as Rc<dyn RollbackEntryModifier>,
            Rc::new(PercentRollbackEntryAdjuster { percent: -12.7 }),
            Rc::new(NameRollbackEntryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.0)]),
            }),
            Rc::new(RollbackEntryEndsAtOffsetAdjuster { weight: 0.77 }),
            Rc::new(FlaggedRollbackEntryAdjuster {
                flags: flags.clone(),
                flag: "rollback-entry-adaptive".to_string(),
                percent: 5.1,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 26581.0,
    }
}
