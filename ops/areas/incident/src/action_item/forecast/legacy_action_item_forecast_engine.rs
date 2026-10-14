use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy action item forecast calculation.
pub trait LegacyActionItemForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyActionItemForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemForecastMinimumOffsetAdjuster {
    pub weight: f64,
}

impl LegacyActionItemForecastAdjuster for LegacyActionItemForecastMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyActionItemForecast, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyActionItemForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyActionItemForecastAdjuster for NameLegacyActionItemForecastAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyActionItemForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyActionItemForecastAdjuster {
    pub step: f64,
}

impl LegacyActionItemForecastAdjuster for RoundingLegacyActionItemForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyActionItemForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyActionItemForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyActionItemForecastAdjuster for TieredLegacyActionItemForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyActionItemForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyActionItemForecastAdjuster {
    pub cap: f64,
}

impl LegacyActionItemForecastAdjuster for CapLegacyActionItemForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyActionItemForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy action item forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyActionItemForecastEngine {
    pub steps: Vec<Rc<dyn LegacyActionItemForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyActionItemForecastEngine {
    pub fn compute(&mut self, item: &LegacyActionItemForecast) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyActionItemForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyActionItemForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_action_item_forecast_engine() -> LegacyActionItemForecastEngine {
    LegacyActionItemForecastEngine {
        steps: vec![
            Rc::new(LegacyActionItemForecastMinimumOffsetAdjuster { weight: 0.96 })
                as Rc<dyn LegacyActionItemForecastAdjuster>,
            Rc::new(NameLegacyActionItemForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.93), ("central".to_string(), 0.88)]),
            }),
            Rc::new(RoundingLegacyActionItemForecastAdjuster { step: 0.05 }),
            Rc::new(TieredLegacyActionItemForecastAdjuster {
                bounds: vec![2671.0, 2680.0, 4939.0, 8414.0],
                rates: vec![1.14, 1.14, 1.02, 1.19],
            }),
            Rc::new(CapLegacyActionItemForecastAdjuster { cap: 44885.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 90236.6,
    }
}
