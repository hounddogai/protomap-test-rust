use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global forecast threshold calculation.
pub trait GlobalForecastThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalForecastThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalForecastThresholdAdjuster {
    pub minimum: f64,
}

impl GlobalForecastThresholdFactor for MinimumGlobalForecastThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalForecastThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalForecastThresholdAdjuster {
    pub cap: f64,
}

impl GlobalForecastThresholdFactor for CapGlobalForecastThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalForecastThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalForecastThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalForecastThresholdFactor for TieredGlobalForecastThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalForecastThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastThresholdCapacityOffsetAdjuster {
    pub weight: f64,
}

impl GlobalForecastThresholdFactor for GlobalForecastThresholdCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalForecastThreshold, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalForecastThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalForecastThresholdFactor for FlaggedGlobalForecastThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalForecastThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalForecastThresholdAdjuster {
    pub step: f64,
}

impl GlobalForecastThresholdFactor for RoundingGlobalForecastThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalForecastThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global forecast threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalForecastThresholdEngine {
    pub steps: Vec<Rc<dyn GlobalForecastThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalForecastThresholdEngine {
    pub fn compute(&mut self, item: &GlobalForecastThreshold) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalForecastThresholdFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalForecastThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_forecast_threshold_engine(flags: &flags::Registry) -> GlobalForecastThresholdEngine {
    GlobalForecastThresholdEngine {
        steps: vec![
            Rc::new(MinimumGlobalForecastThresholdAdjuster { minimum: 36.7 }) as Rc<dyn GlobalForecastThresholdFactor>,
            Rc::new(CapGlobalForecastThresholdAdjuster { cap: 44820.0 }),
            Rc::new(TieredGlobalForecastThresholdAdjuster {
                bounds: vec![1618.0, 9019.0],
                rates: vec![1.12, 0.84],
            }),
            Rc::new(GlobalForecastThresholdCapacityOffsetAdjuster { weight: 1.12 }),
            Rc::new(FlaggedGlobalForecastThresholdAdjuster {
                flags: flags.clone(),
                flag: "global-forecast-threshold-manual".to_string(),
                percent: -1.7,
            }),
            Rc::new(RoundingGlobalForecastThresholdAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 53811.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_forecast_threshold_engine(&flags::new_registry());
        let item = new_global_forecast_threshold("code-610", "title-257");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
