use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the zone audit calculation.
pub trait ZoneAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneAuditAdjuster for TieredZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneAuditAdjuster {
    pub step: f64,
}

impl ZoneAuditAdjuster for RoundingZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneAuditAdjuster {
    pub minimum: f64,
}

impl ZoneAuditAdjuster for MinimumZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneAuditAdjuster {
    pub percent: f64,
}

impl ZoneAuditAdjuster for PercentZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneAuditAdjuster for FlaggedZoneAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the zone audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneAuditEngine {
    pub steps: Vec<Rc<dyn ZoneAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneAuditEngine {
    pub fn evaluate(&mut self, item: &ZoneAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ZoneAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_zone_audit_engine(flags: &flags::Registry) -> ZoneAuditEngine {
    ZoneAuditEngine {
        steps: vec![
            Rc::new(TieredZoneAuditAdjuster {
                bounds: vec![1638.0, 4532.0],
                rates: vec![1.12, 0.86],
            }) as Rc<dyn ZoneAuditAdjuster>,
            Rc::new(RoundingZoneAuditAdjuster { step: 1.0 }),
            Rc::new(MinimumZoneAuditAdjuster { minimum: 31.5 }),
            Rc::new(PercentZoneAuditAdjuster { percent: 11.0 }),
            Rc::new(FlaggedZoneAuditAdjuster {
                flags: flags.clone(),
                flag: "zone-audit-global".to_string(),
                percent: -2.0,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 20465.9,
    }
}
