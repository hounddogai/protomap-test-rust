use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollout forecast calculation.
pub trait RolloutForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutForecastAdjuster {
    pub percent: f64,
}

impl RolloutForecastModifier for PercentRolloutForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRolloutForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutForecastModifier for TitleRolloutForecastAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RolloutForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutForecastAdjuster {
    pub minimum: f64,
}

impl RolloutForecastModifier for MinimumRolloutForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutForecastAdjuster {
    pub step: f64,
}

impl RolloutForecastModifier for RoundingRolloutForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rollout forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutForecastEngine {
    pub steps: Vec<Rc<dyn RolloutForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutForecastEngine {
    pub fn compute(&mut self, item: &RolloutForecast) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollout_forecast_engine() -> RolloutForecastEngine {
    RolloutForecastEngine {
        steps: vec![
            Rc::new(PercentRolloutForecastAdjuster { percent: 23.0 }) as Rc<dyn RolloutForecastModifier>,
            Rc::new(TitleRolloutForecastAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.16),
                    ("west".to_string(), 0.71),
                    ("central".to_string(), 1.03),
                ]),
            }),
            Rc::new(MinimumRolloutForecastAdjuster { minimum: 15.8 }),
            Rc::new(RoundingRolloutForecastAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 21639.7,
    }
}
