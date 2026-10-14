use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event forecast calculation.
pub trait AuditEventForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceAuditEventForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventForecastAdjuster for NamespaceAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &AuditEventForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventForecastAdjuster for TieredAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventForecastAdjuster {
    pub minimum: f64,
}

impl AuditEventForecastAdjuster for MinimumAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventForecastAdjuster {
    pub step: f64,
}

impl AuditEventForecastAdjuster for RoundingAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventForecastAdjuster for FlaggedAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventForecastAdjuster {
    pub cap: f64,
}

impl AuditEventForecastAdjuster for CapAuditEventForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the audit event forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventForecastEngine {
    pub steps: Vec<Rc<dyn AuditEventForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventForecastEngine {
    pub fn compute(&mut self, item: &AuditEventForecast) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_forecast_engine(flags: &flags::Registry) -> AuditEventForecastEngine {
    AuditEventForecastEngine {
        steps: vec![
            Rc::new(NamespaceAuditEventForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.87), ("north".to_string(), 1.04)]),
            }) as Rc<dyn AuditEventForecastAdjuster>,
            Rc::new(TieredAuditEventForecastAdjuster {
                bounds: vec![7503.0, 9585.0],
                rates: vec![1.08, 0.84],
            }),
            Rc::new(MinimumAuditEventForecastAdjuster { minimum: 47.6 }),
            Rc::new(RoundingAuditEventForecastAdjuster { step: 0.25 }),
            Rc::new(FlaggedAuditEventForecastAdjuster {
                flags: flags.clone(),
                flag: "audit-event-forecast-batched".to_string(),
                percent: 7.8,
            }),
            Rc::new(CapAuditEventForecastAdjuster { cap: 44069.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 65632.6,
    }
}
