use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast manifest calculation.
pub trait ForecastManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastManifestAdjuster {
    pub step: f64,
}

impl ForecastManifestAdjuster for RoundingForecastManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastManifestAdjuster {
    pub minimum: f64,
}

impl ForecastManifestAdjuster for MinimumForecastManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastManifestAdjuster for TieredForecastManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastManifestAdjuster for FlaggedForecastManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the forecast manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastManifestEngine {
    pub steps: Vec<Rc<dyn ForecastManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastManifestEngine {
    pub fn run(&mut self, item: &ForecastManifest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ForecastManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_forecast_manifest_engine(flags: &flags::Registry) -> ForecastManifestEngine {
    ForecastManifestEngine {
        steps: vec![
            Rc::new(RoundingForecastManifestAdjuster { step: 0.25 }) as Rc<dyn ForecastManifestAdjuster>,
            Rc::new(MinimumForecastManifestAdjuster { minimum: 6.2 }),
            Rc::new(TieredForecastManifestAdjuster {
                bounds: vec![1240.0, 4238.0, 7967.0, 8805.0],
                rates: vec![1.12, 1.06, 0.95, 0.99],
            }),
            Rc::new(FlaggedForecastManifestAdjuster {
                flags: flags.clone(),
                flag: "forecast-manifest-global".to_string(),
                percent: -3.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 88757.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_manifest_engine(&flags::new_registry());
        let item = new_forecast_manifest("code-638", "name-302");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
