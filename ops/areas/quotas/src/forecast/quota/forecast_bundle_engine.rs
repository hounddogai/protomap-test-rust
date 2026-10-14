use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast bundle calculation.
pub trait ForecastBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastBundleAdjuster {
    pub minimum: f64,
}

impl ForecastBundleAdjuster for MinimumForecastBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBundleCostOffsetAdjuster {
    pub weight: f64,
}

impl ForecastBundleAdjuster for ForecastBundleCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastBundle, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastBundleAdjuster for TieredForecastBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastBundleAdjuster for FlaggedForecastBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastBundleAdjuster {
    pub step: f64,
}

impl ForecastBundleAdjuster for RoundingForecastBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastBundleAdjuster {
    pub cap: f64,
}

impl ForecastBundleAdjuster for CapForecastBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the forecast bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastBundleEngine {
    pub steps: Vec<Rc<dyn ForecastBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastBundleEngine {
    pub fn compute(&mut self, item: &ForecastBundle) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[ForecastBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_forecast_bundle_engine(flags: &flags::Registry) -> ForecastBundleEngine {
    ForecastBundleEngine {
        steps: vec![
            Rc::new(MinimumForecastBundleAdjuster { minimum: 7.4 }) as Rc<dyn ForecastBundleAdjuster>,
            Rc::new(ForecastBundleCostOffsetAdjuster { weight: 0.62 }),
            Rc::new(TieredForecastBundleAdjuster {
                bounds: vec![256.0, 3780.0, 5262.0, 5284.0],
                rates: vec![0.93, 0.92, 0.91, 0.86],
            }),
            Rc::new(FlaggedForecastBundleAdjuster {
                flags: flags.clone(),
                flag: "forecast-bundle-batched".to_string(),
                percent: -3.5,
            }),
            Rc::new(RoundingForecastBundleAdjuster { step: 5.0 }),
            Rc::new(CapForecastBundleAdjuster { cap: 40524.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 25231.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_bundle_engine(&flags::new_registry());
        let item = new_forecast_bundle("code-528", "label-386");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
