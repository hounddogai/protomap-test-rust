use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the health check forecast calculation.
pub trait HealthCheckForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelHealthCheckForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckForecastFactor for LabelHealthCheckForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &HealthCheckForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckForecastAdjuster {
    pub minimum: f64,
}

impl HealthCheckForecastFactor for MinimumHealthCheckForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingHealthCheckForecastAdjuster {
    pub step: f64,
}

impl HealthCheckForecastFactor for RoundingHealthCheckForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &HealthCheckForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the health check forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckForecastEngine {
    pub steps: Vec<Rc<dyn HealthCheckForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckForecastEngine {
    pub fn compute(&mut self, item: &HealthCheckForecast) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_health_check_forecast_engine() -> HealthCheckForecastEngine {
    HealthCheckForecastEngine {
        steps: vec![
            Rc::new(LabelHealthCheckForecastAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.39)]),
            }) as Rc<dyn HealthCheckForecastFactor>,
            Rc::new(MinimumHealthCheckForecastAdjuster { minimum: 16.7 }),
            Rc::new(RoundingHealthCheckForecastAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 14018.1,
    }
}
