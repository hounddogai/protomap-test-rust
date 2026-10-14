use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted segment forecast calculation.
pub trait WeightedSegmentForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedSegmentForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedSegmentForecastAdjuster {
    pub percent: f64,
}

impl WeightedSegmentForecastStep for PercentWeightedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedSegmentForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedSegmentForecastAdjuster {
    pub minimum: f64,
}

impl WeightedSegmentForecastStep for MinimumWeightedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedSegmentForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedSegmentForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedSegmentForecastStep for TieredWeightedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedSegmentForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedSegmentForecastAdjuster {
    pub step: f64,
}

impl WeightedSegmentForecastStep for RoundingWeightedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedSegmentForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastMaximumOffsetAdjuster {
    pub weight: f64,
}

impl WeightedSegmentForecastStep for WeightedSegmentForecastMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedSegmentForecast, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleWeightedSegmentForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedSegmentForecastStep for TitleWeightedSegmentForecastAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &WeightedSegmentForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted segment forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedSegmentForecastEngine {
    pub steps: Vec<Rc<dyn WeightedSegmentForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedSegmentForecastEngine {
    pub fn price(&mut self, item: &WeightedSegmentForecast) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[WeightedSegmentForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_segment_forecast_engine() -> WeightedSegmentForecastEngine {
    WeightedSegmentForecastEngine {
        steps: vec![
            Rc::new(PercentWeightedSegmentForecastAdjuster { percent: 20.5 }) as Rc<dyn WeightedSegmentForecastStep>,
            Rc::new(MinimumWeightedSegmentForecastAdjuster { minimum: 6.8 }),
            Rc::new(TieredWeightedSegmentForecastAdjuster {
                bounds: vec![2469.0, 4829.0, 9790.0],
                rates: vec![1.14, 1.05, 0.95],
            }),
            Rc::new(RoundingWeightedSegmentForecastAdjuster { step: 0.05 }),
            Rc::new(WeightedSegmentForecastMaximumOffsetAdjuster { weight: 1.8 }),
            Rc::new(TitleWeightedSegmentForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.76), ("north".to_string(), 0.8)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 43730.0,
    }
}
