use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention forecast calculation.
pub trait RetentionForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionForecastMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RetentionForecastAdjuster for RetentionForecastMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionForecast, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionForecastAdjuster {
    pub cap: f64,
}

impl RetentionForecastAdjuster for CapRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRetentionForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionForecastAdjuster for LabelRetentionForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RetentionForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the retention forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionForecastEngine {
    pub steps: Vec<Rc<dyn RetentionForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionForecastEngine {
    pub fn evaluate(&mut self, item: &RetentionForecast) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[RetentionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_forecast_engine() -> RetentionForecastEngine {
    RetentionForecastEngine {
        steps: vec![
            Rc::new(RetentionForecastMaximumOffsetAdjuster { weight: 0.22 }) as Rc<dyn RetentionForecastAdjuster>,
            Rc::new(CapRetentionForecastAdjuster { cap: 36840.0 }),
            Rc::new(LabelRetentionForecastAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.79)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 68326.5,
    }
}
