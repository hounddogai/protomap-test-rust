use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict isolation forecast calculation.
pub trait StrictIsolationForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictIsolationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictIsolationForecastAdjuster {
    pub cap: f64,
}

impl StrictIsolationForecastStep for CapStrictIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictIsolationForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictIsolationForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictIsolationForecastStep for TieredStrictIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictIsolationForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictIsolationForecastAdjuster {
    pub minimum: f64,
}

impl StrictIsolationForecastStep for MinimumStrictIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictIsolationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictIsolationForecastAdjuster {
    pub step: f64,
}

impl StrictIsolationForecastStep for RoundingStrictIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictIsolationForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictIsolationForecastAdjuster {
    pub percent: f64,
}

impl StrictIsolationForecastStep for PercentStrictIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictIsolationForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict isolation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictIsolationForecastEngine {
    pub steps: Vec<Rc<dyn StrictIsolationForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictIsolationForecastEngine {
    pub fn evaluate(&mut self, item: &StrictIsolationForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictIsolationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_isolation_forecast_engine() -> StrictIsolationForecastEngine {
    StrictIsolationForecastEngine {
        steps: vec![
            Rc::new(CapStrictIsolationForecastAdjuster { cap: 13222.0 }) as Rc<dyn StrictIsolationForecastStep>,
            Rc::new(TieredStrictIsolationForecastAdjuster {
                bounds: vec![3153.0, 5049.0, 6003.0],
                rates: vec![1.18, 1.16, 0.84],
            }),
            Rc::new(MinimumStrictIsolationForecastAdjuster { minimum: 2.3 }),
            Rc::new(RoundingStrictIsolationForecastAdjuster { step: 0.1 }),
            Rc::new(PercentStrictIsolationForecastAdjuster { percent: 6.4 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 61656.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_isolation_forecast_engine();
        let item = new_strict_isolation_forecast("code-961", "label-738");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
