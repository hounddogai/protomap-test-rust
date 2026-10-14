use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension forecast calculation.
pub trait SuspensionForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionForecastAdjuster {
    pub percent: f64,
}

impl SuspensionForecastAdjuster for PercentSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionForecastAdjuster for TieredSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionForecastAdjuster {
    pub step: f64,
}

impl SuspensionForecastAdjuster for RoundingSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the suspension forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionForecastEngine {
    pub steps: Vec<Rc<dyn SuspensionForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionForecastEngine {
    pub fn evaluate(&mut self, item: &SuspensionForecast) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_forecast_engine() -> SuspensionForecastEngine {
    SuspensionForecastEngine {
        steps: vec![
            Rc::new(PercentSuspensionForecastAdjuster { percent: -20.4 }) as Rc<dyn SuspensionForecastAdjuster>,
            Rc::new(TieredSuspensionForecastAdjuster {
                bounds: vec![6183.0, 7748.0],
                rates: vec![0.83, 0.91],
            }),
            Rc::new(RoundingSuspensionForecastAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 74886.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_forecast_engine();
        let item = new_suspension_forecast("code-434", "name-560");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
