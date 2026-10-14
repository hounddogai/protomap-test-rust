use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional forecast bundle calculation.
pub trait RegionalForecastBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalForecastBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalForecastBundleAdjuster {
    pub minimum: f64,
}

impl RegionalForecastBundleModifier for MinimumRegionalForecastBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalForecastBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalForecastBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalForecastBundleModifier for TieredRegionalForecastBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalForecastBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalForecastBundleTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RegionalForecastBundleModifier for RegionalForecastBundleTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalForecastBundle, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the regional forecast bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalForecastBundleEngine {
    pub steps: Vec<Rc<dyn RegionalForecastBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalForecastBundleEngine {
    pub fn run(&mut self, item: &RegionalForecastBundle) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalForecastBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_forecast_bundle_engine() -> RegionalForecastBundleEngine {
    RegionalForecastBundleEngine {
        steps: vec![
            Rc::new(MinimumRegionalForecastBundleAdjuster { minimum: 19.2 }) as Rc<dyn RegionalForecastBundleModifier>,
            Rc::new(TieredRegionalForecastBundleAdjuster {
                bounds: vec![2759.0, 3160.0],
                rates: vec![1.05, 1.04],
            }),
            Rc::new(RegionalForecastBundleTtlSecondsOffsetAdjuster { weight: 1.75 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 3047.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_forecast_bundle_engine();
        let item = new_regional_forecast_bundle("code-334", "label-677");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
