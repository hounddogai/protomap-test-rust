use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering forecast calculation.
pub trait PeeringForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringForecastAdjuster {
    pub minimum: f64,
}

impl PeeringForecastModifier for MinimumPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringForecastModifier for FlaggedPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PeeringForecastModifier for PeeringForecastEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringForecast, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringForecastAdjuster {
    pub percent: f64,
}

impl PeeringForecastModifier for PercentPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringForecastAdjuster {
    pub step: f64,
}

impl PeeringForecastModifier for RoundingPeeringForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the peering forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringForecastEngine {
    pub steps: Vec<Rc<dyn PeeringForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringForecastEngine {
    pub fn compute(&mut self, item: &PeeringForecast) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_peering_forecast_engine(flags: &flags::Registry) -> PeeringForecastEngine {
    PeeringForecastEngine {
        steps: vec![
            Rc::new(MinimumPeeringForecastAdjuster { minimum: 49.3 }) as Rc<dyn PeeringForecastModifier>,
            Rc::new(FlaggedPeeringForecastAdjuster {
                flags: flags.clone(),
                flag: "peering-forecast-rolling".to_string(),
                percent: -6.0,
            }),
            Rc::new(PeeringForecastEndsAtOffsetAdjuster { weight: 0.89 }),
            Rc::new(PercentPeeringForecastAdjuster { percent: -18.6 }),
            Rc::new(RoundingPeeringForecastAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 92955.2,
    }
}
