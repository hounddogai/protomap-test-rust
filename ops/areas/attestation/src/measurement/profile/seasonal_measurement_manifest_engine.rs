use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal measurement manifest calculation.
pub trait SeasonalMeasurementManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalMeasurementManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalMeasurementManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalMeasurementManifestModifier for TieredSeasonalMeasurementManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalMeasurementManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalMeasurementManifestAdjuster {
    pub cap: f64,
}

impl SeasonalMeasurementManifestModifier for CapSeasonalMeasurementManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalMeasurementManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSeasonalMeasurementManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalMeasurementManifestModifier for TitleSeasonalMeasurementManifestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SeasonalMeasurementManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal measurement manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalMeasurementManifestEngine {
    pub steps: Vec<Rc<dyn SeasonalMeasurementManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalMeasurementManifestEngine {
    pub fn price(&mut self, item: &SeasonalMeasurementManifest) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalMeasurementManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_measurement_manifest_engine() -> SeasonalMeasurementManifestEngine {
    SeasonalMeasurementManifestEngine {
        steps: vec![
            Rc::new(TieredSeasonalMeasurementManifestAdjuster {
                bounds: vec![3883.0, 4947.0],
                rates: vec![1.09, 0.87],
            }) as Rc<dyn SeasonalMeasurementManifestModifier>,
            Rc::new(CapSeasonalMeasurementManifestAdjuster { cap: 15866.0 }),
            Rc::new(TitleSeasonalMeasurementManifestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.03), ("south".to_string(), 0.85)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 9679.1,
    }
}
