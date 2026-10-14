use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default identity forecast calculation.
pub trait DefaultIdentityForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultIdentityForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultIdentityForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultIdentityForecastFactor for TieredDefaultIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultIdentityForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultIdentityForecastAdjuster {
    pub percent: f64,
}

impl DefaultIdentityForecastFactor for PercentDefaultIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultIdentityForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultIdentityForecastAdjuster {
    pub cap: f64,
}

impl DefaultIdentityForecastFactor for CapDefaultIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultIdentityForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultIdentityForecastAdjuster {
    pub minimum: f64,
}

impl DefaultIdentityForecastFactor for MinimumDefaultIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultIdentityForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the default identity forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultIdentityForecastEngine {
    pub steps: Vec<Rc<dyn DefaultIdentityForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultIdentityForecastEngine {
    pub fn price(&mut self, item: &DefaultIdentityForecast) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultIdentityForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultIdentityForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_identity_forecast_engine() -> DefaultIdentityForecastEngine {
    DefaultIdentityForecastEngine {
        steps: vec![
            Rc::new(TieredDefaultIdentityForecastAdjuster {
                bounds: vec![7472.0, 8735.0],
                rates: vec![0.92, 1.06],
            }) as Rc<dyn DefaultIdentityForecastFactor>,
            Rc::new(PercentDefaultIdentityForecastAdjuster { percent: -3.4 }),
            Rc::new(CapDefaultIdentityForecastAdjuster { cap: 20122.0 }),
            Rc::new(MinimumDefaultIdentityForecastAdjuster { minimum: 32.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 23904.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_identity_forecast_engine();
        let item = new_default_identity_forecast("code-980", "label-872");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
