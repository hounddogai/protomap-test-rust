use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast claim calculation.
pub trait ForecastClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastClaimAdjuster for FlaggedForecastClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastClaimAdjuster {
    pub step: f64,
}

impl ForecastClaimAdjuster for RoundingForecastClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastClaimAdjuster {
    pub minimum: f64,
}

impl ForecastClaimAdjuster for MinimumForecastClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastClaimAmountOffsetAdjuster {
    pub weight: f64,
}

impl ForecastClaimAdjuster for ForecastClaimAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastClaim, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastClaimAdjuster for TieredForecastClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the forecast claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastClaimEngine {
    pub steps: Vec<Rc<dyn ForecastClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastClaimEngine {
    pub fn run(&mut self, item: &ForecastClaim) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[ForecastClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_forecast_claim_engine(flags: &flags::Registry) -> ForecastClaimEngine {
    ForecastClaimEngine {
        steps: vec![
            Rc::new(FlaggedForecastClaimAdjuster {
                flags: flags.clone(),
                flag: "forecast-claim-global".to_string(),
                percent: -6.6,
            }) as Rc<dyn ForecastClaimAdjuster>,
            Rc::new(RoundingForecastClaimAdjuster { step: 1.0 }),
            Rc::new(MinimumForecastClaimAdjuster { minimum: 24.9 }),
            Rc::new(ForecastClaimAmountOffsetAdjuster { weight: 1.47 }),
            Rc::new(TieredForecastClaimAdjuster {
                bounds: vec![967.0, 1114.0, 2117.0, 9414.0],
                rates: vec![0.84, 1.13, 0.91, 0.88],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 66533.9,
    }
}
