use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event item calculation.
pub trait AuditEventItemStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventItemStep for TieredAuditEventItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceAuditEventItemAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventItemStep for NamespaceAuditEventItemAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &AuditEventItem, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventItemAdjuster {
    pub step: f64,
}

impl AuditEventItemStep for RoundingAuditEventItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the audit event item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventItemEngine {
    pub steps: Vec<Rc<dyn AuditEventItemStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventItemEngine {
    pub fn compute(&mut self, item: &AuditEventItem) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_item_engine() -> AuditEventItemEngine {
    AuditEventItemEngine {
        steps: vec![
            Rc::new(TieredAuditEventItemAdjuster {
                bounds: vec![3166.0, 4858.0, 7971.0, 7995.0],
                rates: vec![1.03, 0.9, 1.04, 0.83],
            }) as Rc<dyn AuditEventItemStep>,
            Rc::new(NamespaceAuditEventItemAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.04), ("west".to_string(), 1.24)]),
            }),
            Rc::new(RoundingAuditEventItemAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 9998.0,
    }
}
