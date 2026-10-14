use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global audit event override calculation.
pub trait GlobalAuditEventOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAuditEventOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalAuditEventOverrideAdjuster {
    pub percent: f64,
}

impl GlobalAuditEventOverrideModifier for PercentGlobalAuditEventOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalAuditEventOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalAuditEventOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalAuditEventOverrideModifier for TieredGlobalAuditEventOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalAuditEventOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalAuditEventOverrideAdjuster {
    pub minimum: f64,
}

impl GlobalAuditEventOverrideModifier for MinimumGlobalAuditEventOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalAuditEventOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global audit event override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventOverrideEngine {
    pub steps: Vec<Rc<dyn GlobalAuditEventOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAuditEventOverrideEngine {
    pub fn compute(&mut self, item: &GlobalAuditEventOverride) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAuditEventOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_audit_event_override_engine() -> GlobalAuditEventOverrideEngine {
    GlobalAuditEventOverrideEngine {
        steps: vec![
            Rc::new(PercentGlobalAuditEventOverrideAdjuster { percent: 4.5 })
                as Rc<dyn GlobalAuditEventOverrideModifier>,
            Rc::new(TieredGlobalAuditEventOverrideAdjuster {
                bounds: vec![801.0, 8491.0, 8761.0],
                rates: vec![1.01, 0.89, 1.12],
            }),
            Rc::new(MinimumGlobalAuditEventOverrideAdjuster { minimum: 30.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 95388.4,
    }
}
