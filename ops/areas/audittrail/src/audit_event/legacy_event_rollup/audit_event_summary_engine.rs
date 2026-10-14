use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event summary calculation.
pub trait AuditEventSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventSummaryModifier for FlaggedAuditEventSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAuditEventSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventSummaryModifier for NameAuditEventSummaryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &AuditEventSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventSummaryAdjuster {
    pub cap: f64,
}

impl AuditEventSummaryModifier for CapAuditEventSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSummaryBurstOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventSummaryModifier for AuditEventSummaryBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventSummary, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventSummaryAdjuster {
    pub percent: f64,
}

impl AuditEventSummaryModifier for PercentAuditEventSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventSummaryModifier for TieredAuditEventSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the audit event summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventSummaryEngine {
    pub steps: Vec<Rc<dyn AuditEventSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventSummaryEngine {
    pub fn compute(&mut self, item: &AuditEventSummary) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_summary_engine(flags: &flags::Registry) -> AuditEventSummaryEngine {
    AuditEventSummaryEngine {
        steps: vec![
            Rc::new(FlaggedAuditEventSummaryAdjuster {
                flags: flags.clone(),
                flag: "audit-event-summary-rolling".to_string(),
                percent: -8.1,
            }) as Rc<dyn AuditEventSummaryModifier>,
            Rc::new(NameAuditEventSummaryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.84), ("south".to_string(), 1.15)]),
            }),
            Rc::new(CapAuditEventSummaryAdjuster { cap: 3242.0 }),
            Rc::new(AuditEventSummaryBurstOffsetAdjuster { weight: 1.56 }),
            Rc::new(PercentAuditEventSummaryAdjuster { percent: 3.9 }),
            Rc::new(TieredAuditEventSummaryAdjuster {
                bounds: vec![4210.0, 4852.0, 6186.0, 9015.0],
                rates: vec![0.88, 0.89, 0.97, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 32645.0,
    }
}
