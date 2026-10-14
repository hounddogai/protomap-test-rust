use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched forecast manifest calculation.
pub trait BatchedForecastManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedForecastManifest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedForecastManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedForecastManifestAdjuster for FlaggedBatchedForecastManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedForecastManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBatchedForecastManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedForecastManifestAdjuster for CountryBatchedForecastManifestAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BatchedForecastManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedForecastManifestAdjuster {
    pub minimum: f64,
}

impl BatchedForecastManifestAdjuster for MinimumBatchedForecastManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedForecastManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedForecastManifestAdjuster {
    pub step: f64,
}

impl BatchedForecastManifestAdjuster for RoundingBatchedForecastManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedForecastManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched forecast manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedForecastManifestEngine {
    pub steps: Vec<Rc<dyn BatchedForecastManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedForecastManifestEngine {
    pub fn compute(&mut self, item: &BatchedForecastManifest) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedForecastManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_forecast_manifest_engine(flags: &flags::Registry) -> BatchedForecastManifestEngine {
    BatchedForecastManifestEngine {
        steps: vec![
            Rc::new(FlaggedBatchedForecastManifestAdjuster {
                flags: flags.clone(),
                flag: "batched-forecast-manifest-strict".to_string(),
                percent: 6.1,
            }) as Rc<dyn BatchedForecastManifestAdjuster>,
            Rc::new(CountryBatchedForecastManifestAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.35),
                    ("south".to_string(), 1.07),
                    ("central".to_string(), 1.29),
                ]),
            }),
            Rc::new(MinimumBatchedForecastManifestAdjuster { minimum: 24.5 }),
            Rc::new(RoundingBatchedForecastManifestAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 68790.0,
    }
}
