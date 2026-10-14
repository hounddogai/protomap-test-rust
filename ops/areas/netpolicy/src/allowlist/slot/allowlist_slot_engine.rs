use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist slot calculation.
pub trait AllowlistSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistSlotModifier for TieredAllowlistSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistSlotAdjuster {
    pub minimum: f64,
}

impl AllowlistSlotModifier for MinimumAllowlistSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistSlotModifier for FlaggedAllowlistSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistSlotAdjuster {
    pub cap: f64,
}

impl AllowlistSlotModifier for CapAllowlistSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSlotRevisionOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistSlotModifier for AllowlistSlotRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistSlot, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the allowlist slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistSlotEngine {
    pub steps: Vec<Rc<dyn AllowlistSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistSlotEngine {
    pub fn run(&mut self, item: &AllowlistSlot) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_allowlist_slot_engine(flags: &flags::Registry) -> AllowlistSlotEngine {
    AllowlistSlotEngine {
        steps: vec![
            Rc::new(TieredAllowlistSlotAdjuster {
                bounds: vec![847.0, 1355.0],
                rates: vec![0.89, 0.92],
            }) as Rc<dyn AllowlistSlotModifier>,
            Rc::new(MinimumAllowlistSlotAdjuster { minimum: 25.1 }),
            Rc::new(FlaggedAllowlistSlotAdjuster {
                flags: flags.clone(),
                flag: "allowlist-slot-scheduled".to_string(),
                percent: 1.8,
            }),
            Rc::new(CapAllowlistSlotAdjuster { cap: 8201.0 }),
            Rc::new(AllowlistSlotRevisionOffsetAdjuster { weight: 0.4 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 47240.5,
    }
}
