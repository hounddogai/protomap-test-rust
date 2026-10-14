use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event quota calculation.
pub trait AuditEventQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventQuotaAdjuster {
    pub step: f64,
}

impl AuditEventQuotaFactor for RoundingAuditEventQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventQuotaFactor for FlaggedAuditEventQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventQuotaAdjuster {
    pub minimum: f64,
}

impl AuditEventQuotaFactor for MinimumAuditEventQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventQuotaAdjuster {
    pub percent: f64,
}

impl AuditEventQuotaFactor for PercentAuditEventQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the audit event quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventQuotaEngine {
    pub steps: Vec<Rc<dyn AuditEventQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventQuotaEngine {
    pub fn evaluate(&mut self, item: &AuditEventQuota) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_audit_event_quota_engine(flags: &flags::Registry) -> AuditEventQuotaEngine {
    AuditEventQuotaEngine {
        steps: vec![
            Rc::new(RoundingAuditEventQuotaAdjuster { step: 0.25 }) as Rc<dyn AuditEventQuotaFactor>,
            Rc::new(FlaggedAuditEventQuotaAdjuster {
                flags: flags.clone(),
                flag: "audit-event-quota-seasonal".to_string(),
                percent: 0.0,
            }),
            Rc::new(MinimumAuditEventQuotaAdjuster { minimum: 35.6 }),
            Rc::new(PercentAuditEventQuotaAdjuster { percent: -23.7 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 87006.4,
    }
}
