use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered tenant entry calculation.
pub trait TieredTenantEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredTenantEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredTenantEntryAdjuster {
    pub cap: f64,
}

impl TieredTenantEntryModifier for CapTieredTenantEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredTenantEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantEntryCeilingOffsetAdjuster {
    pub weight: f64,
}

impl TieredTenantEntryModifier for TieredTenantEntryCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &TieredTenantEntry, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredTenantEntryAdjuster {
    pub minimum: f64,
}

impl TieredTenantEntryModifier for MinimumTieredTenantEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredTenantEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredTenantEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredTenantEntryModifier for FlaggedTieredTenantEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredTenantEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredTenantEntryAdjuster {
    pub percent: f64,
}

impl TieredTenantEntryModifier for PercentTieredTenantEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredTenantEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredTenantEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredTenantEntryModifier for TieredTieredTenantEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredTenantEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered tenant entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredTenantEntryEngine {
    pub steps: Vec<Rc<dyn TieredTenantEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredTenantEntryEngine {
    pub fn price(&mut self, item: &TieredTenantEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredTenantEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_tenant_entry_engine(flags: &flags::Registry) -> TieredTenantEntryEngine {
    TieredTenantEntryEngine {
        steps: vec![
            Rc::new(CapTieredTenantEntryAdjuster { cap: 15664.0 }) as Rc<dyn TieredTenantEntryModifier>,
            Rc::new(TieredTenantEntryCeilingOffsetAdjuster { weight: 1.63 }),
            Rc::new(MinimumTieredTenantEntryAdjuster { minimum: 35.1 }),
            Rc::new(FlaggedTieredTenantEntryAdjuster {
                flags: flags.clone(),
                flag: "tiered-tenant-entry-tiered".to_string(),
                percent: 5.7,
            }),
            Rc::new(PercentTieredTenantEntryAdjuster { percent: 10.5 }),
            Rc::new(TieredTieredTenantEntryAdjuster {
                bounds: vec![373.0, 4492.0],
                rates: vec![1.12, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 84415.3,
    }
}
