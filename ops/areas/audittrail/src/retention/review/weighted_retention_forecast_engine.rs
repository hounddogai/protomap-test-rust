use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted retention forecast calculation.
pub trait WeightedRetentionForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRetentionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRetentionForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRetentionForecastAdjuster for FlaggedWeightedRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRetentionForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRetentionForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRetentionForecastAdjuster for TieredWeightedRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRetentionForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionForecastBudgetOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRetentionForecastAdjuster for WeightedRetentionForecastBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRetentionForecast, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedRetentionForecastAdjuster {
    pub step: f64,
}

impl WeightedRetentionForecastAdjuster for RoundingWeightedRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedRetentionForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRetentionForecastAdjuster {
    pub cap: f64,
}

impl WeightedRetentionForecastAdjuster for CapWeightedRetentionForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRetentionForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted retention forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRetentionForecastEngine {
    pub steps: Vec<Rc<dyn WeightedRetentionForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRetentionForecastEngine {
    pub fn evaluate(&mut self, item: &WeightedRetentionForecast) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRetentionForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRetentionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_retention_forecast_engine(flags: &flags::Registry) -> WeightedRetentionForecastEngine {
    WeightedRetentionForecastEngine {
        steps: vec![
            Rc::new(FlaggedWeightedRetentionForecastAdjuster {
                flags: flags.clone(),
                flag: "weighted-retention-forecast-rolling".to_string(),
                percent: -8.2,
            }) as Rc<dyn WeightedRetentionForecastAdjuster>,
            Rc::new(TieredWeightedRetentionForecastAdjuster {
                bounds: vec![1671.0, 5965.0, 6432.0],
                rates: vec![1.18, 1.03, 0.83],
            }),
            Rc::new(WeightedRetentionForecastBudgetOffsetAdjuster { weight: 1.38 }),
            Rc::new(RoundingWeightedRetentionForecastAdjuster { step: 0.25 }),
            Rc::new(CapWeightedRetentionForecastAdjuster { cap: 4206.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 1312.2,
    }
}
