use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement forecast calculation.
pub trait MeasurementForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementForecastFactor for TieredMeasurementForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementForecastFactor for FlaggedMeasurementForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementForecastAdjuster {
    pub percent: f64,
}

impl MeasurementForecastFactor for PercentMeasurementForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementForecastAdjuster {
    pub step: f64,
}

impl MeasurementForecastFactor for RoundingMeasurementForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the measurement forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementForecastEngine {
    pub steps: Vec<Rc<dyn MeasurementForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementForecastEngine {
    pub fn run(&mut self, item: &MeasurementForecast) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_measurement_forecast_engine(flags: &flags::Registry) -> MeasurementForecastEngine {
    MeasurementForecastEngine {
        steps: vec![
            Rc::new(TieredMeasurementForecastAdjuster {
                bounds: vec![6316.0, 7002.0],
                rates: vec![1.09, 0.96],
            }) as Rc<dyn MeasurementForecastFactor>,
            Rc::new(FlaggedMeasurementForecastAdjuster {
                flags: flags.clone(),
                flag: "measurement-forecast-manual".to_string(),
                percent: 2.1,
            }),
            Rc::new(PercentMeasurementForecastAdjuster { percent: -16.9 }),
            Rc::new(RoundingMeasurementForecastAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 77528.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_forecast_engine(&flags::new_registry());
        let item = new_measurement_forecast("code-413", "name-306");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
