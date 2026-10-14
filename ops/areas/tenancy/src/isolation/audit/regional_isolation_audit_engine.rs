use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional isolation audit calculation.
pub trait RegionalIsolationAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalIsolationAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalIsolationAuditAdjuster {
    pub cap: f64,
}

impl RegionalIsolationAuditFactor for CapRegionalIsolationAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalIsolationAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalIsolationAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalIsolationAuditFactor for NameRegionalIsolationAuditAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalIsolationAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalIsolationAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalIsolationAuditFactor for FlaggedRegionalIsolationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalIsolationAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional isolation audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalIsolationAuditEngine {
    pub steps: Vec<Rc<dyn RegionalIsolationAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalIsolationAuditEngine {
    pub fn compute(&mut self, item: &RegionalIsolationAudit) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalIsolationAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_isolation_audit_engine(flags: &flags::Registry) -> RegionalIsolationAuditEngine {
    RegionalIsolationAuditEngine {
        steps: vec![
            Rc::new(CapRegionalIsolationAuditAdjuster { cap: 15811.0 }) as Rc<dyn RegionalIsolationAuditFactor>,
            Rc::new(NameRegionalIsolationAuditAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.12),
                    ("east".to_string(), 1.34),
                    ("online".to_string(), 1.36),
                ]),
            }),
            Rc::new(FlaggedRegionalIsolationAuditAdjuster {
                flags: flags.clone(),
                flag: "regional-isolation-audit-regional".to_string(),
                percent: -5.0,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 76044.3,
    }
}
