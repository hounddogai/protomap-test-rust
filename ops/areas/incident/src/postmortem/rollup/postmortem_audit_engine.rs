use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem audit calculation.
pub trait PostmortemAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemAudit, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemAuditModifier for FlaggedPostmortemAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemAuditAdjuster {
    pub percent: f64,
}

impl PostmortemAuditModifier for PercentPostmortemAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemAuditAdjuster {
    pub step: f64,
}

impl PostmortemAuditModifier for RoundingPostmortemAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemAuditAdjuster {
    pub minimum: f64,
}

impl PostmortemAuditModifier for MinimumPostmortemAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the postmortem audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemAuditEngine {
    pub steps: Vec<Rc<dyn PostmortemAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemAuditEngine {
    pub fn evaluate(&mut self, item: &PostmortemAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PostmortemAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_postmortem_audit_engine(flags: &flags::Registry) -> PostmortemAuditEngine {
    PostmortemAuditEngine {
        steps: vec![
            Rc::new(FlaggedPostmortemAuditAdjuster {
                flags: flags.clone(),
                flag: "postmortem-audit-primary".to_string(),
                percent: 9.2,
            }) as Rc<dyn PostmortemAuditModifier>,
            Rc::new(PercentPostmortemAuditAdjuster { percent: -22.1 }),
            Rc::new(RoundingPostmortemAuditAdjuster { step: 0.25 }),
            Rc::new(MinimumPostmortemAuditAdjuster { minimum: 11.3 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 90502.9,
    }
}
