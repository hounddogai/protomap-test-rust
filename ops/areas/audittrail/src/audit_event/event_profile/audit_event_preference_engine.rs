use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event preference calculation.
pub trait AuditEventPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventPreferenceAdjuster {
    pub minimum: f64,
}

impl AuditEventPreferenceStep for MinimumAuditEventPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventPreferenceStep for FlaggedAuditEventPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventPreferenceAdjuster {
    pub step: f64,
}

impl AuditEventPreferenceStep for RoundingAuditEventPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventPreferenceAdjuster {
    pub cap: f64,
}

impl AuditEventPreferenceStep for CapAuditEventPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the audit event preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventPreferenceEngine {
    pub steps: Vec<Rc<dyn AuditEventPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventPreferenceEngine {
    pub fn compute(&mut self, item: &AuditEventPreference) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_preference_engine(flags: &flags::Registry) -> AuditEventPreferenceEngine {
    AuditEventPreferenceEngine {
        steps: vec![
            Rc::new(MinimumAuditEventPreferenceAdjuster { minimum: 4.9 }) as Rc<dyn AuditEventPreferenceStep>,
            Rc::new(FlaggedAuditEventPreferenceAdjuster {
                flags: flags.clone(),
                flag: "audit-event-preference-strict".to_string(),
                percent: 7.9,
            }),
            Rc::new(RoundingAuditEventPreferenceAdjuster { step: 0.25 }),
            Rc::new(CapAuditEventPreferenceAdjuster { cap: 28775.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 2221.2,
    }
}
