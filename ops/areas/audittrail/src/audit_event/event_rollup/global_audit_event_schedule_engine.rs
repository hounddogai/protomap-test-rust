use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global audit event schedule calculation.
pub trait GlobalAuditEventScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAuditEventSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalAuditEventScheduleAdjuster {
    pub cap: f64,
}

impl GlobalAuditEventScheduleAdjuster for CapGlobalAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalAuditEventSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventScheduleStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalAuditEventScheduleAdjuster for GlobalAuditEventScheduleStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalAuditEventSchedule, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalAuditEventScheduleAdjuster {
    pub step: f64,
}

impl GlobalAuditEventScheduleAdjuster for RoundingGlobalAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalAuditEventSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalAuditEventScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalAuditEventScheduleAdjuster for TieredGlobalAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalAuditEventSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalAuditEventScheduleAdjuster {
    pub percent: f64,
}

impl GlobalAuditEventScheduleAdjuster for PercentGlobalAuditEventScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalAuditEventSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global audit event schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventScheduleEngine {
    pub steps: Vec<Rc<dyn GlobalAuditEventScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAuditEventScheduleEngine {
    pub fn evaluate(&mut self, item: &GlobalAuditEventSchedule) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAuditEventSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_audit_event_schedule_engine() -> GlobalAuditEventScheduleEngine {
    GlobalAuditEventScheduleEngine {
        steps: vec![
            Rc::new(CapGlobalAuditEventScheduleAdjuster { cap: 45913.0 }) as Rc<dyn GlobalAuditEventScheduleAdjuster>,
            Rc::new(GlobalAuditEventScheduleStartsAtOffsetAdjuster { weight: 0.41 }),
            Rc::new(RoundingGlobalAuditEventScheduleAdjuster { step: 0.1 }),
            Rc::new(TieredGlobalAuditEventScheduleAdjuster {
                bounds: vec![2774.0, 4008.0, 4231.0, 6039.0],
                rates: vec![0.88, 0.91, 1.01, 1.09],
            }),
            Rc::new(PercentGlobalAuditEventScheduleAdjuster { percent: -17.6 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 75119.7,
    }
}
