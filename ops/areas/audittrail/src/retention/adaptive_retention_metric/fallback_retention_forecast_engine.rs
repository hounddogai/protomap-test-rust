use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback retention forecast calculation.
pub trait FallbackRetentionForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRetentionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackRetentionForecastAdjuster {
    pub step: f64,
}

impl FallbackRetentionForecastAdjuster for RoundingFallbackRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackRetentionForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRetentionForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRetentionForecastAdjuster for FlaggedFallbackRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRetentionForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRetentionForecastAdjuster {
    pub cap: f64,
}

impl FallbackRetentionForecastAdjuster for CapFallbackRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRetentionForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRetentionForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRetentionForecastAdjuster for TieredFallbackRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRetentionForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeFallbackRetentionForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRetentionForecastAdjuster for ScopeFallbackRetentionForecastAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &FallbackRetentionForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRetentionForecastAdjuster {
    pub minimum: f64,
}

impl FallbackRetentionForecastAdjuster for MinimumFallbackRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRetentionForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback retention forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRetentionForecastEngine {
    pub steps: Vec<Rc<dyn FallbackRetentionForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRetentionForecastEngine {
    pub fn compute(&mut self, item: &FallbackRetentionForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackRetentionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_retention_forecast_engine(flags: &flags::Registry) -> FallbackRetentionForecastEngine {
    FallbackRetentionForecastEngine {
        steps: vec![
            Rc::new(RoundingFallbackRetentionForecastAdjuster { step: 0.25 })
                as Rc<dyn FallbackRetentionForecastAdjuster>,
            Rc::new(FlaggedFallbackRetentionForecastAdjuster {
                flags: flags.clone(),
                flag: "fallback-retention-forecast-legacy".to_string(),
                percent: 3.0,
            }),
            Rc::new(CapFallbackRetentionForecastAdjuster { cap: 8920.0 }),
            Rc::new(TieredFallbackRetentionForecastAdjuster {
                bounds: vec![858.0, 1526.0, 5355.0],
                rates: vec![1.06, 1.0, 0.86],
            }),
            Rc::new(ScopeFallbackRetentionForecastAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.77),
                    ("east".to_string(), 1.17),
                    ("online".to_string(), 0.89),
                ]),
            }),
            Rc::new(MinimumFallbackRetentionForecastAdjuster { minimum: 32.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 64026.9,
    }
}
