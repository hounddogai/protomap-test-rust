use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item forecast calculation.
pub trait ActionItemForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemForecastAdjuster {
    pub percent: f64,
}

impl ActionItemForecastStep for PercentActionItemForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemForecastAdjuster {
    pub cap: f64,
}

impl ActionItemForecastStep for CapActionItemForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemForecastStep for TieredActionItemForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemForecastStep for FlaggedActionItemForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the action item forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemForecastEngine {
    pub steps: Vec<Rc<dyn ActionItemForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemForecastEngine {
    pub fn compute(&mut self, item: &ActionItemForecast) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_action_item_forecast_engine(flags: &flags::Registry) -> ActionItemForecastEngine {
    ActionItemForecastEngine {
        steps: vec![
            Rc::new(PercentActionItemForecastAdjuster { percent: -10.6 }) as Rc<dyn ActionItemForecastStep>,
            Rc::new(CapActionItemForecastAdjuster { cap: 8389.0 }),
            Rc::new(TieredActionItemForecastAdjuster {
                bounds: vec![965.0, 8598.0],
                rates: vec![0.96, 0.94],
            }),
            Rc::new(FlaggedActionItemForecastAdjuster {
                flags: flags.clone(),
                flag: "action-item-forecast-tiered".to_string(),
                percent: 5.1,
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 59097.8,
    }
}
