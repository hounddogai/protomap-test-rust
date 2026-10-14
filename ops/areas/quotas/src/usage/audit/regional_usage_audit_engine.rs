use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional usage audit calculation.
pub trait RegionalUsageAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalUsageAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalUsageAuditAdjuster {
    pub minimum: f64,
}

impl RegionalUsageAuditAdjuster for MinimumRegionalUsageAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalUsageAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageAuditQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RegionalUsageAuditAdjuster for RegionalUsageAuditQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalUsageAudit, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalUsageAuditAdjuster {
    pub step: f64,
}

impl RegionalUsageAuditAdjuster for RoundingRegionalUsageAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalUsageAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalUsageAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalUsageAuditAdjuster for TitleRegionalUsageAuditAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalUsageAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalUsageAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalUsageAuditAdjuster for FlaggedRegionalUsageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalUsageAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional usage audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalUsageAuditEngine {
    pub steps: Vec<Rc<dyn RegionalUsageAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalUsageAuditEngine {
    pub fn price(&mut self, item: &RegionalUsageAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RegionalUsageAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_usage_audit_engine(flags: &flags::Registry) -> RegionalUsageAuditEngine {
    RegionalUsageAuditEngine {
        steps: vec![
            Rc::new(MinimumRegionalUsageAuditAdjuster { minimum: 42.3 }) as Rc<dyn RegionalUsageAuditAdjuster>,
            Rc::new(RegionalUsageAuditQuantityOffsetAdjuster { weight: 0.13 }),
            Rc::new(RoundingRegionalUsageAuditAdjuster { step: 5.0 }),
            Rc::new(TitleRegionalUsageAuditAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.22), ("online".to_string(), 1.26)]),
            }),
            Rc::new(FlaggedRegionalUsageAuditAdjuster {
                flags: flags.clone(),
                flag: "regional-usage-audit-weighted".to_string(),
                percent: -2.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 57326.2,
    }
}
