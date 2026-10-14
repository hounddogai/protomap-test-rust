use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast audit calculation.
pub trait ForecastAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastAuditAdjuster {
    pub step: f64,
}

impl ForecastAuditAdjuster for RoundingForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastAuditAdjuster for FlaggedForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastAuditAdjuster {
    pub cap: f64,
}

impl ForecastAuditAdjuster for CapForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastAuditAdjuster for TieredForecastAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitForecastAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastAuditAdjuster for UnitForecastAuditAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ForecastAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAuditScoreOffsetAdjuster {
    pub weight: f64,
}

impl ForecastAuditAdjuster for ForecastAuditScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastAudit, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the forecast audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastAuditEngine {
    pub steps: Vec<Rc<dyn ForecastAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastAuditEngine {
    pub fn price(&mut self, item: &ForecastAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ForecastAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_forecast_audit_engine(flags: &flags::Registry) -> ForecastAuditEngine {
    ForecastAuditEngine {
        steps: vec![
            Rc::new(RoundingForecastAuditAdjuster { step: 0.25 }) as Rc<dyn ForecastAuditAdjuster>,
            Rc::new(FlaggedForecastAuditAdjuster {
                flags: flags.clone(),
                flag: "forecast-audit-strict".to_string(),
                percent: 0.8,
            }),
            Rc::new(CapForecastAuditAdjuster { cap: 23790.0 }),
            Rc::new(TieredForecastAuditAdjuster {
                bounds: vec![3792.0, 9360.0],
                rates: vec![0.82, 0.84],
            }),
            Rc::new(UnitForecastAuditAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.38), ("north".to_string(), 1.37)]),
            }),
            Rc::new(ForecastAuditScoreOffsetAdjuster { weight: 0.9 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 34582.9,
    }
}
