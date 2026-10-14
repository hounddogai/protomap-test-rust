use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary canary forecast calculation.
pub trait PrimaryCanaryForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryCanaryForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryCanaryForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryCanaryForecastStep for FlaggedPrimaryCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryCanaryForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryCanaryForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryCanaryForecastStep for TieredPrimaryCanaryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryCanaryForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryForecastCeilingOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryCanaryForecastStep for PrimaryCanaryForecastCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryCanaryForecast, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the primary canary forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryCanaryForecastEngine {
    pub steps: Vec<Rc<dyn PrimaryCanaryForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryCanaryForecastEngine {
    pub fn price(&mut self, item: &PrimaryCanaryForecast) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryCanaryForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryCanaryForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_canary_forecast_engine(flags: &flags::Registry) -> PrimaryCanaryForecastEngine {
    PrimaryCanaryForecastEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryCanaryForecastAdjuster {
                flags: flags.clone(),
                flag: "primary-canary-forecast-primary".to_string(),
                percent: -7.7,
            }) as Rc<dyn PrimaryCanaryForecastStep>,
            Rc::new(TieredPrimaryCanaryForecastAdjuster {
                bounds: vec![5177.0, 7479.0],
                rates: vec![0.86, 1.02],
            }),
            Rc::new(PrimaryCanaryForecastCeilingOffsetAdjuster { weight: 1.94 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 10039.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_canary_forecast_engine(&flags::new_registry());
        let item = new_primary_canary_forecast("code-411", "label-456");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
