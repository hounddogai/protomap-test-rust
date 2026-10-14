use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive forecast score calculation.
pub trait AdaptiveForecastScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveForecastScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAdaptiveForecastScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveForecastScoreModifier for LocaleAdaptiveForecastScoreAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AdaptiveForecastScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveForecastScoreAdjuster {
    pub cap: f64,
}

impl AdaptiveForecastScoreModifier for CapAdaptiveForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveForecastScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastScoreRatioOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveForecastScoreModifier for AdaptiveForecastScoreRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveForecastScore, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveForecastScoreAdjuster {
    pub minimum: f64,
}

impl AdaptiveForecastScoreModifier for MinimumAdaptiveForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveForecastScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive forecast score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveForecastScoreEngine {
    pub steps: Vec<Rc<dyn AdaptiveForecastScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveForecastScoreEngine {
    pub fn evaluate(&mut self, item: &AdaptiveForecastScore) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveForecastScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_forecast_score_engine() -> AdaptiveForecastScoreEngine {
    AdaptiveForecastScoreEngine {
        steps: vec![
            Rc::new(LocaleAdaptiveForecastScoreAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.93), ("retail".to_string(), 1.27)]),
            }) as Rc<dyn AdaptiveForecastScoreModifier>,
            Rc::new(CapAdaptiveForecastScoreAdjuster { cap: 39231.0 }),
            Rc::new(AdaptiveForecastScoreRatioOffsetAdjuster { weight: 0.85 }),
            Rc::new(MinimumAdaptiveForecastScoreAdjuster { minimum: 32.8 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 74187.5,
    }
}
