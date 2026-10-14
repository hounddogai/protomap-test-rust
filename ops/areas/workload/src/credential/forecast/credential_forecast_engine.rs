use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the credential forecast calculation.
pub trait CredentialForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialForecastAdjuster for TieredCredentialForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialForecastAdjuster {
    pub cap: f64,
}

impl CredentialForecastAdjuster for CapCredentialForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialForecastAdjuster for FlaggedCredentialForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialForecastAdjuster {
    pub minimum: f64,
}

impl CredentialForecastAdjuster for MinimumCredentialForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the credential forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialForecastEngine {
    pub steps: Vec<Rc<dyn CredentialForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialForecastEngine {
    pub fn evaluate(&mut self, item: &CredentialForecast) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CredentialForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_credential_forecast_engine(flags: &flags::Registry) -> CredentialForecastEngine {
    CredentialForecastEngine {
        steps: vec![
            Rc::new(TieredCredentialForecastAdjuster {
                bounds: vec![2192.0, 2207.0, 5919.0],
                rates: vec![1.17, 0.89, 1.05],
            }) as Rc<dyn CredentialForecastAdjuster>,
            Rc::new(CapCredentialForecastAdjuster { cap: 7150.0 }),
            Rc::new(FlaggedCredentialForecastAdjuster {
                flags: flags.clone(),
                flag: "credential-forecast-default".to_string(),
                percent: 6.7,
            }),
            Rc::new(MinimumCredentialForecastAdjuster { minimum: 49.1 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 50100.0,
    }
}
