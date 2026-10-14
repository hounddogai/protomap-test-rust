use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the freeze forecast calculation.
pub trait FreezeForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeForecastAdjuster {
    pub minimum: f64,
}

impl FreezeForecastModifier for MinimumFreezeForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeForecastAdjuster {
    pub percent: f64,
}

impl FreezeForecastModifier for PercentFreezeForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentFreezeForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeForecastModifier for SegmentFreezeForecastAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &FreezeForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeForecastAdjuster {
    pub cap: f64,
}

impl FreezeForecastModifier for CapFreezeForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeForecastModifier for TieredFreezeForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the freeze forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeForecastEngine {
    pub steps: Vec<Rc<dyn FreezeForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeForecastEngine {
    pub fn price(&mut self, item: &FreezeForecast) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_freeze_forecast_engine() -> FreezeForecastEngine {
    FreezeForecastEngine {
        steps: vec![
            Rc::new(MinimumFreezeForecastAdjuster { minimum: 3.0 }) as Rc<dyn FreezeForecastModifier>,
            Rc::new(PercentFreezeForecastAdjuster { percent: -23.1 }),
            Rc::new(SegmentFreezeForecastAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.23)]),
            }),
            Rc::new(CapFreezeForecastAdjuster { cap: 38092.0 }),
            Rc::new(TieredFreezeForecastAdjuster {
                bounds: vec![3526.0, 4707.0, 7096.0, 8519.0],
                rates: vec![0.8, 1.19, 1.15, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 92672.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_forecast_engine();
        let item = new_freeze_forecast("code-771", "title-469");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
