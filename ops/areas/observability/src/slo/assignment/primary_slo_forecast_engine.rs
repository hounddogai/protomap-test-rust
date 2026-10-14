use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary slo forecast calculation.
pub trait PrimarySloForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySloForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimarySloForecastAdjuster {
    pub minimum: f64,
}

impl PrimarySloForecastModifier for MinimumPrimarySloForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimarySloForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPrimarySloForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimarySloForecastModifier for SegmentPrimarySloForecastAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PrimarySloForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimarySloForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimarySloForecastModifier for TieredPrimarySloForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimarySloForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimarySloForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimarySloForecastModifier for FlaggedPrimarySloForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimarySloForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimarySloForecastAdjuster {
    pub percent: f64,
}

impl PrimarySloForecastModifier for PercentPrimarySloForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimarySloForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary slo forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySloForecastEngine {
    pub steps: Vec<Rc<dyn PrimarySloForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySloForecastEngine {
    pub fn run(&mut self, item: &PrimarySloForecast) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimarySloForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySloForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_slo_forecast_engine(flags: &flags::Registry) -> PrimarySloForecastEngine {
    PrimarySloForecastEngine {
        steps: vec![
            Rc::new(MinimumPrimarySloForecastAdjuster { minimum: 47.4 }) as Rc<dyn PrimarySloForecastModifier>,
            Rc::new(SegmentPrimarySloForecastAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.06),
                    ("south".to_string(), 0.77),
                    ("retail".to_string(), 1.12),
                ]),
            }),
            Rc::new(TieredPrimarySloForecastAdjuster {
                bounds: vec![1003.0, 6122.0],
                rates: vec![0.89, 1.03],
            }),
            Rc::new(FlaggedPrimarySloForecastAdjuster {
                flags: flags.clone(),
                flag: "primary-slo-forecast-legacy".to_string(),
                percent: -9.0,
            }),
            Rc::new(PercentPrimarySloForecastAdjuster { percent: 10.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 85479.3,
    }
}
