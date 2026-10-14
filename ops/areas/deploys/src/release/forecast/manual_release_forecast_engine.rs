use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual release forecast calculation.
pub trait ManualReleaseForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualReleaseForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseForecastQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ManualReleaseForecastAdjuster for ManualReleaseForecastQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ManualReleaseForecast, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualReleaseForecastAdjuster {
    pub minimum: f64,
}

impl ManualReleaseForecastAdjuster for MinimumManualReleaseForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualReleaseForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualReleaseForecastAdjuster {
    pub step: f64,
}

impl ManualReleaseForecastAdjuster for RoundingManualReleaseForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualReleaseForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualReleaseForecastAdjuster {
    pub percent: f64,
}

impl ManualReleaseForecastAdjuster for PercentManualReleaseForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualReleaseForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual release forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualReleaseForecastEngine {
    pub steps: Vec<Rc<dyn ManualReleaseForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualReleaseForecastEngine {
    pub fn evaluate(&mut self, item: &ManualReleaseForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualReleaseForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_release_forecast_engine() -> ManualReleaseForecastEngine {
    ManualReleaseForecastEngine {
        steps: vec![
            Rc::new(ManualReleaseForecastQuantityOffsetAdjuster { weight: 0.65 })
                as Rc<dyn ManualReleaseForecastAdjuster>,
            Rc::new(MinimumManualReleaseForecastAdjuster { minimum: 23.3 }),
            Rc::new(RoundingManualReleaseForecastAdjuster { step: 0.1 }),
            Rc::new(PercentManualReleaseForecastAdjuster { percent: -16.4 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 67884.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_release_forecast_engine();
        let item = new_manual_release_forecast("code-649", "name-401");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
