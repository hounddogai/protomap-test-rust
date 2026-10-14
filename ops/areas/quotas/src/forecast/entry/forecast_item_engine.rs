use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast item calculation.
pub trait ForecastItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastItemAdjuster {
    pub minimum: f64,
}

impl ForecastItemAdjuster for MinimumForecastItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItemRateOffsetAdjuster {
    pub weight: f64,
}

impl ForecastItemAdjuster for ForecastItemRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastItem, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastItemAdjuster {
    pub cap: f64,
}

impl ForecastItemAdjuster for CapForecastItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastItemAdjuster {
    pub percent: f64,
}

impl ForecastItemAdjuster for PercentForecastItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the forecast item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastItemEngine {
    pub steps: Vec<Rc<dyn ForecastItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastItemEngine {
    pub fn run(&mut self, item: &ForecastItem) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[ForecastItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_forecast_item_engine() -> ForecastItemEngine {
    ForecastItemEngine {
        steps: vec![
            Rc::new(MinimumForecastItemAdjuster { minimum: 44.0 }) as Rc<dyn ForecastItemAdjuster>,
            Rc::new(ForecastItemRateOffsetAdjuster { weight: 1.96 }),
            Rc::new(CapForecastItemAdjuster { cap: 12573.0 }),
            Rc::new(PercentForecastItemAdjuster { percent: 17.4 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 91362.4,
    }
}
