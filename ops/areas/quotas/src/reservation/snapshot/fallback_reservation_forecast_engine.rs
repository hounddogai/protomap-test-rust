use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback reservation forecast calculation.
pub trait FallbackReservationForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackReservationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackReservationForecastAdjuster {
    pub cap: f64,
}

impl FallbackReservationForecastFactor for CapFallbackReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackReservationForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationForecastCapacityOffsetAdjuster {
    pub weight: f64,
}

impl FallbackReservationForecastFactor for FallbackReservationForecastCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackReservationForecast, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackReservationForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackReservationForecastFactor for TieredFallbackReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackReservationForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackReservationForecastAdjuster {
    pub minimum: f64,
}

impl FallbackReservationForecastFactor for MinimumFallbackReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackReservationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackReservationForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackReservationForecastFactor for FlaggedFallbackReservationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackReservationForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback reservation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackReservationForecastEngine {
    pub steps: Vec<Rc<dyn FallbackReservationForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackReservationForecastEngine {
    pub fn compute(&mut self, item: &FallbackReservationForecast) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackReservationForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackReservationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_reservation_forecast_engine(flags: &flags::Registry) -> FallbackReservationForecastEngine {
    FallbackReservationForecastEngine {
        steps: vec![
            Rc::new(CapFallbackReservationForecastAdjuster { cap: 708.0 }) as Rc<dyn FallbackReservationForecastFactor>,
            Rc::new(FallbackReservationForecastCapacityOffsetAdjuster { weight: 1.62 }),
            Rc::new(TieredFallbackReservationForecastAdjuster {
                bounds: vec![3167.0, 4091.0, 5999.0],
                rates: vec![1.15, 1.04, 1.01],
            }),
            Rc::new(MinimumFallbackReservationForecastAdjuster { minimum: 17.7 }),
            Rc::new(FlaggedFallbackReservationForecastAdjuster {
                flags: flags.clone(),
                flag: "fallback-reservation-forecast-weighted".to_string(),
                percent: 7.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 59395.7,
    }
}
