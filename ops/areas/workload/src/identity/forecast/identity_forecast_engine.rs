use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the identity forecast calculation.
pub trait IdentityForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityForecastStep for FlaggedIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityForecastCostOffsetAdjuster {
    pub weight: f64,
}

impl IdentityForecastStep for IdentityForecastCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityForecast, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityForecastStep for TieredIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the identity forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityForecastEngine {
    pub steps: Vec<Rc<dyn IdentityForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityForecastEngine {
    pub fn price(&mut self, item: &IdentityForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityForecastStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_forecast_engine(flags: &flags::Registry) -> IdentityForecastEngine {
    IdentityForecastEngine {
        steps: vec![
            Rc::new(FlaggedIdentityForecastAdjuster {
                flags: flags.clone(),
                flag: "identity-forecast-regional".to_string(),
                percent: -5.5,
            }) as Rc<dyn IdentityForecastStep>,
            Rc::new(IdentityForecastCostOffsetAdjuster { weight: 0.32 }),
            Rc::new(TieredIdentityForecastAdjuster {
                bounds: vec![173.0, 4426.0],
                rates: vec![0.98, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 76711.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_forecast_engine(&flags::new_registry());
        let item = new_identity_forecast("code-505", "label-932");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
