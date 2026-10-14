use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release forecast calculation.
pub trait ReleaseForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseForecastFactor for TieredReleaseForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseForecastAdjuster {
    pub cap: f64,
}

impl ReleaseForecastFactor for CapReleaseForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseForecastFactor for FlaggedReleaseForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseForecastAdjuster {
    pub percent: f64,
}

impl ReleaseForecastFactor for PercentReleaseForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitReleaseForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseForecastFactor for UnitReleaseForecastAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ReleaseForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the release forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseForecastEngine {
    pub steps: Vec<Rc<dyn ReleaseForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseForecastEngine {
    pub fn run(&mut self, item: &ReleaseForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReleaseForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_forecast_engine(flags: &flags::Registry) -> ReleaseForecastEngine {
    ReleaseForecastEngine {
        steps: vec![
            Rc::new(TieredReleaseForecastAdjuster {
                bounds: vec![5205.0, 8233.0],
                rates: vec![1.14, 0.99],
            }) as Rc<dyn ReleaseForecastFactor>,
            Rc::new(CapReleaseForecastAdjuster { cap: 5669.0 }),
            Rc::new(FlaggedReleaseForecastAdjuster {
                flags: flags.clone(),
                flag: "release-forecast-regional".to_string(),
                percent: -0.2,
            }),
            Rc::new(PercentReleaseForecastAdjuster { percent: -19.3 }),
            Rc::new(UnitReleaseForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.77), ("south".to_string(), 1.14)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 12496.9,
    }
}
