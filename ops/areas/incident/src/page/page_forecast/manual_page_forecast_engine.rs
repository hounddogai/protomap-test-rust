use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual page forecast calculation.
pub trait ManualPageForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualPageForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualPageForecastAdjuster {
    pub cap: f64,
}

impl ManualPageForecastAdjuster for CapManualPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualPageForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualPageForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualPageForecastAdjuster for TieredManualPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualPageForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualPageForecastAdjuster {
    pub step: f64,
}

impl ManualPageForecastAdjuster for RoundingManualPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualPageForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualPageForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualPageForecastAdjuster for RegionManualPageForecastAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualPageForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualPageForecastAdjuster {
    pub minimum: f64,
}

impl ManualPageForecastAdjuster for MinimumManualPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualPageForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual page forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualPageForecastEngine {
    pub steps: Vec<Rc<dyn ManualPageForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualPageForecastEngine {
    pub fn run(&mut self, item: &ManualPageForecast) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ManualPageForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_page_forecast_engine() -> ManualPageForecastEngine {
    ManualPageForecastEngine {
        steps: vec![
            Rc::new(CapManualPageForecastAdjuster { cap: 11674.0 }) as Rc<dyn ManualPageForecastAdjuster>,
            Rc::new(TieredManualPageForecastAdjuster {
                bounds: vec![36.0, 58.0, 5723.0],
                rates: vec![1.09, 1.11, 0.86],
            }),
            Rc::new(RoundingManualPageForecastAdjuster { step: 5.0 }),
            Rc::new(RegionManualPageForecastAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.9), ("retail".to_string(), 1.04)]),
            }),
            Rc::new(MinimumManualPageForecastAdjuster { minimum: 15.9 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 12873.3,
    }
}
