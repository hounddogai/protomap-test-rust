use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the audit event threshold calculation.
pub trait AuditEventThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAuditEventThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventThresholdAdjuster for UnitAuditEventThresholdAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &AuditEventThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventThresholdAdjuster for FlaggedAuditEventThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventThresholdAdjuster for TieredAuditEventThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventThresholdAdjuster {
    pub percent: f64,
}

impl AuditEventThresholdAdjuster for PercentAuditEventThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventThresholdAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventThresholdAdjuster for AuditEventThresholdAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventThreshold, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventThresholdAdjuster {
    pub minimum: f64,
}

impl AuditEventThresholdAdjuster for MinimumAuditEventThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the audit event threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventThresholdEngine {
    pub steps: Vec<Rc<dyn AuditEventThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventThresholdEngine {
    pub fn evaluate(&mut self, item: &AuditEventThreshold) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_audit_event_threshold_engine(flags: &flags::Registry) -> AuditEventThresholdEngine {
    AuditEventThresholdEngine {
        steps: vec![
            Rc::new(UnitAuditEventThresholdAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.15)]),
            }) as Rc<dyn AuditEventThresholdAdjuster>,
            Rc::new(FlaggedAuditEventThresholdAdjuster {
                flags: flags.clone(),
                flag: "audit-event-threshold-adaptive".to_string(),
                percent: -3.2,
            }),
            Rc::new(TieredAuditEventThresholdAdjuster {
                bounds: vec![1047.0, 2631.0, 3979.0, 6859.0],
                rates: vec![0.81, 0.87, 0.92, 0.93],
            }),
            Rc::new(PercentAuditEventThresholdAdjuster { percent: 4.7 }),
            Rc::new(AuditEventThresholdAttemptsOffsetAdjuster { weight: 1.15 }),
            Rc::new(MinimumAuditEventThresholdAdjuster { minimum: 13.8 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 24899.5,
    }
}
