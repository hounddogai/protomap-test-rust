use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global runbook audit calculation.
pub trait GlobalRunbookAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalRunbookAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalRunbookAuditAdjuster {
    pub step: f64,
}

impl GlobalRunbookAuditStep for RoundingGlobalRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalRunbookAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalRunbookAuditAdjuster {
    pub minimum: f64,
}

impl GlobalRunbookAuditStep for MinimumGlobalRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalRunbookAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalRunbookAuditAdjuster {
    pub percent: f64,
}

impl GlobalRunbookAuditStep for PercentGlobalRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalRunbookAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalRunbookAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalRunbookAuditStep for TieredGlobalRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalRunbookAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalRunbookAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalRunbookAuditStep for FlaggedGlobalRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalRunbookAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl GlobalRunbookAuditStep for GlobalRunbookAuditDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalRunbookAudit, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

/// Runs the global runbook audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalRunbookAuditEngine {
    pub steps: Vec<Rc<dyn GlobalRunbookAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalRunbookAuditEngine {
    pub fn run(&mut self, item: &GlobalRunbookAudit) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[GlobalRunbookAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_runbook_audit_engine(flags: &flags::Registry) -> GlobalRunbookAuditEngine {
    GlobalRunbookAuditEngine {
        steps: vec![
            Rc::new(RoundingGlobalRunbookAuditAdjuster { step: 0.25 }) as Rc<dyn GlobalRunbookAuditStep>,
            Rc::new(MinimumGlobalRunbookAuditAdjuster { minimum: 45.0 }),
            Rc::new(PercentGlobalRunbookAuditAdjuster { percent: 18.7 }),
            Rc::new(TieredGlobalRunbookAuditAdjuster {
                bounds: vec![1834.0, 7512.0],
                rates: vec![1.07, 0.83],
            }),
            Rc::new(FlaggedGlobalRunbookAuditAdjuster {
                flags: flags.clone(),
                flag: "global-runbook-audit-manual".to_string(),
                percent: -5.8,
            }),
            Rc::new(GlobalRunbookAuditDurationMinutesOffsetAdjuster { weight: 1.12 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 85455.8,
    }
}
