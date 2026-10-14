use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy runbook audit calculation.
pub trait LegacyRunbookAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRunbookAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRunbookAuditAdjuster {
    pub minimum: f64,
}

impl LegacyRunbookAuditFactor for MinimumLegacyRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRunbookAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRunbookAuditMaximumOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRunbookAuditFactor for LegacyRunbookAuditMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRunbookAudit, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyRunbookAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyRunbookAuditFactor for FlaggedLegacyRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyRunbookAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyRunbookAuditAdjuster {
    pub step: f64,
}

impl LegacyRunbookAuditFactor for RoundingLegacyRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyRunbookAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRunbookAuditAdjuster {
    pub percent: f64,
}

impl LegacyRunbookAuditFactor for PercentLegacyRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRunbookAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy runbook audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRunbookAuditEngine {
    pub steps: Vec<Rc<dyn LegacyRunbookAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRunbookAuditEngine {
    pub fn evaluate(&mut self, item: &LegacyRunbookAudit) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyRunbookAuditFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[LegacyRunbookAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_runbook_audit_engine(flags: &flags::Registry) -> LegacyRunbookAuditEngine {
    LegacyRunbookAuditEngine {
        steps: vec![
            Rc::new(MinimumLegacyRunbookAuditAdjuster { minimum: 48.8 }) as Rc<dyn LegacyRunbookAuditFactor>,
            Rc::new(LegacyRunbookAuditMaximumOffsetAdjuster { weight: 1.13 }),
            Rc::new(FlaggedLegacyRunbookAuditAdjuster {
                flags: flags.clone(),
                flag: "legacy-runbook-audit-default".to_string(),
                percent: 7.5,
            }),
            Rc::new(RoundingLegacyRunbookAuditAdjuster { step: 0.05 }),
            Rc::new(PercentLegacyRunbookAuditAdjuster { percent: 11.6 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 14813.8,
    }
}
