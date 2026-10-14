use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event rollup calculation.
pub trait AuditEventRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventRollupAdjuster {
    pub cap: f64,
}

impl AuditEventRollupFactor for CapAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventRollupAdjuster {
    pub step: f64,
}

impl AuditEventRollupFactor for RoundingAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRollupDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventRollupFactor for AuditEventRollupDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventRollup, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventRollupAdjuster {
    pub minimum: f64,
}

impl AuditEventRollupFactor for MinimumAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeAuditEventRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventRollupFactor for ScopeAuditEventRollupAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &AuditEventRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the audit event rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventRollupEngine {
    pub steps: Vec<Rc<dyn AuditEventRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventRollupEngine {
    pub fn evaluate(&mut self, item: &AuditEventRollup) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_audit_event_rollup_engine() -> AuditEventRollupEngine {
    AuditEventRollupEngine {
        steps: vec![
            Rc::new(CapAuditEventRollupAdjuster { cap: 42863.0 }) as Rc<dyn AuditEventRollupFactor>,
            Rc::new(RoundingAuditEventRollupAdjuster { step: 0.5 }),
            Rc::new(AuditEventRollupDurationMinutesOffsetAdjuster { weight: 0.97 }),
            Rc::new(MinimumAuditEventRollupAdjuster { minimum: 36.1 }),
            Rc::new(ScopeAuditEventRollupAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.73),
                    ("west".to_string(), 1.0),
                    ("north".to_string(), 1.36),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 52008.1,
    }
}
