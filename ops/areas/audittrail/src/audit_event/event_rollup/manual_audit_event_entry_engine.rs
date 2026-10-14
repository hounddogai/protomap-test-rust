use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual audit event entry calculation.
pub trait ManualAuditEventEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualAuditEventEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualAuditEventEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualAuditEventEntryAdjuster for FlaggedManualAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualAuditEventEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualAuditEventEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualAuditEventEntryAdjuster for TieredManualAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualAuditEventEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitManualAuditEventEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualAuditEventEntryAdjuster for UnitManualAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ManualAuditEventEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual audit event entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualAuditEventEntryEngine {
    pub steps: Vec<Rc<dyn ManualAuditEventEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualAuditEventEntryEngine {
    pub fn price(&mut self, item: &ManualAuditEventEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualAuditEventEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_audit_event_entry_engine(flags: &flags::Registry) -> ManualAuditEventEntryEngine {
    ManualAuditEventEntryEngine {
        steps: vec![
            Rc::new(FlaggedManualAuditEventEntryAdjuster {
                flags: flags.clone(),
                flag: "manual-audit-event-entry-legacy".to_string(),
                percent: -4.6,
            }) as Rc<dyn ManualAuditEventEntryAdjuster>,
            Rc::new(TieredManualAuditEventEntryAdjuster {
                bounds: vec![1546.0, 3068.0, 6999.0],
                rates: vec![1.15, 0.96, 0.96],
            }),
            Rc::new(UnitManualAuditEventEntryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.34), ("north".to_string(), 1.34)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 89814.5,
    }
}
