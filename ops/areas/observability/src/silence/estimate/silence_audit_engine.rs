use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence audit calculation.
pub trait SilenceAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceAuditModifier for TieredSilenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceAuditAdjuster {
    pub percent: f64,
}

impl SilenceAuditModifier for PercentSilenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceAuditModifier for FlaggedSilenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSilenceAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceAuditModifier for NameSilenceAuditAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SilenceAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSilenceAuditAdjuster {
    pub cap: f64,
}

impl SilenceAuditModifier for CapSilenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SilenceAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the silence audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceAuditEngine {
    pub steps: Vec<Rc<dyn SilenceAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceAuditEngine {
    pub fn compute(&mut self, item: &SilenceAudit) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[SilenceAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_silence_audit_engine(flags: &flags::Registry) -> SilenceAuditEngine {
    SilenceAuditEngine {
        steps: vec![
            Rc::new(TieredSilenceAuditAdjuster {
                bounds: vec![2346.0, 8201.0],
                rates: vec![0.96, 1.0],
            }) as Rc<dyn SilenceAuditModifier>,
            Rc::new(PercentSilenceAuditAdjuster { percent: -10.5 }),
            Rc::new(FlaggedSilenceAuditAdjuster {
                flags: flags.clone(),
                flag: "silence-audit-batched".to_string(),
                percent: -0.5,
            }),
            Rc::new(NameSilenceAuditAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.2)]),
            }),
            Rc::new(CapSilenceAuditAdjuster { cap: 49132.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 40109.6,
    }
}
