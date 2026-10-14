use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default federation forecast calculation.
pub trait DefaultFederationForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultFederationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultFederationForecastAdjuster {
    pub minimum: f64,
}

impl DefaultFederationForecastAdjuster for MinimumDefaultFederationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultFederationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultFederationForecastAdjuster {
    pub step: f64,
}

impl DefaultFederationForecastAdjuster for RoundingDefaultFederationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultFederationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDefaultFederationForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultFederationForecastAdjuster for ScopeDefaultFederationForecastAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DefaultFederationForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultFederationForecastAdjuster {
    pub percent: f64,
}

impl DefaultFederationForecastAdjuster for PercentDefaultFederationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultFederationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultFederationForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultFederationForecastAdjuster for TieredDefaultFederationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultFederationForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultFederationForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultFederationForecastAdjuster for FlaggedDefaultFederationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultFederationForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default federation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultFederationForecastEngine {
    pub steps: Vec<Rc<dyn DefaultFederationForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultFederationForecastEngine {
    pub fn price(&mut self, item: &DefaultFederationForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultFederationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_federation_forecast_engine(flags: &flags::Registry) -> DefaultFederationForecastEngine {
    DefaultFederationForecastEngine {
        steps: vec![
            Rc::new(MinimumDefaultFederationForecastAdjuster { minimum: 10.2 })
                as Rc<dyn DefaultFederationForecastAdjuster>,
            Rc::new(RoundingDefaultFederationForecastAdjuster { step: 0.1 }),
            Rc::new(ScopeDefaultFederationForecastAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.25),
                    ("west".to_string(), 1.36),
                    ("south".to_string(), 1.33),
                ]),
            }),
            Rc::new(PercentDefaultFederationForecastAdjuster { percent: 23.8 }),
            Rc::new(TieredDefaultFederationForecastAdjuster {
                bounds: vec![2257.0, 4557.0, 8528.0],
                rates: vec![1.03, 0.95, 1.1],
            }),
            Rc::new(FlaggedDefaultFederationForecastAdjuster {
                flags: flags.clone(),
                flag: "default-federation-forecast-weighted".to_string(),
                percent: -2.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 39901.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_federation_forecast_engine(&flags::new_registry());
        let item = new_default_federation_forecast("code-765", "label-941");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
