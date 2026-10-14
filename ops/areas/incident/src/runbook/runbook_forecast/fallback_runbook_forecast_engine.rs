use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback runbook forecast calculation.
pub trait FallbackRunbookForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRunbookForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRunbookForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRunbookForecastFactor for TieredFallbackRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRunbookForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackRunbookForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRunbookForecastFactor for NamespaceFallbackRunbookForecastAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackRunbookForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookForecastCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRunbookForecastFactor for FallbackRunbookForecastCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRunbookForecast, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRunbookForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRunbookForecastFactor for FlaggedFallbackRunbookForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRunbookForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback runbook forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRunbookForecastEngine {
    pub steps: Vec<Rc<dyn FallbackRunbookForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRunbookForecastEngine {
    pub fn compute(&mut self, item: &FallbackRunbookForecast) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRunbookForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_runbook_forecast_engine(flags: &flags::Registry) -> FallbackRunbookForecastEngine {
    FallbackRunbookForecastEngine {
        steps: vec![
            Rc::new(TieredFallbackRunbookForecastAdjuster {
                bounds: vec![1570.0, 8868.0],
                rates: vec![1.01, 0.93],
            }) as Rc<dyn FallbackRunbookForecastFactor>,
            Rc::new(NamespaceFallbackRunbookForecastAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.81),
                    ("west".to_string(), 0.84),
                    ("online".to_string(), 1.28),
                ]),
            }),
            Rc::new(FallbackRunbookForecastCreatedAtOffsetAdjuster { weight: 0.75 }),
            Rc::new(FlaggedFallbackRunbookForecastAdjuster {
                flags: flags.clone(),
                flag: "fallback-runbook-forecast-strict".to_string(),
                percent: -7.0,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 92729.0,
    }
}
