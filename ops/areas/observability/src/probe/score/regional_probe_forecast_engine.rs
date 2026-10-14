use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional probe forecast calculation.
pub trait RegionalProbeForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalProbeForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalProbeForecastAdjuster {
    pub cap: f64,
}

impl RegionalProbeForecastAdjuster for CapRegionalProbeForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalProbeForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalProbeForecastAdjuster {
    pub minimum: f64,
}

impl RegionalProbeForecastAdjuster for MinimumRegionalProbeForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalProbeForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalProbeForecastAdjuster {
    pub step: f64,
}

impl RegionalProbeForecastAdjuster for RoundingRegionalProbeForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalProbeForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional probe forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalProbeForecastEngine {
    pub steps: Vec<Rc<dyn RegionalProbeForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalProbeForecastEngine {
    pub fn run(&mut self, item: &RegionalProbeForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalProbeForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalProbeForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_probe_forecast_engine() -> RegionalProbeForecastEngine {
    RegionalProbeForecastEngine {
        steps: vec![
            Rc::new(CapRegionalProbeForecastAdjuster { cap: 9266.0 }) as Rc<dyn RegionalProbeForecastAdjuster>,
            Rc::new(MinimumRegionalProbeForecastAdjuster { minimum: 28.7 }),
            Rc::new(RoundingRegionalProbeForecastAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 20893.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_probe_forecast_engine();
        let item = new_regional_probe_forecast("code-788", "label-549");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
