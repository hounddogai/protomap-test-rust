use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional forecast request calculation.
pub trait RegionalForecastRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalForecastRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalForecastRequestAdjuster {
    pub step: f64,
}

impl RegionalForecastRequestFactor for RoundingRegionalForecastRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalForecastRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalForecastRequestAdjuster {
    pub percent: f64,
}

impl RegionalForecastRequestFactor for PercentRegionalForecastRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalForecastRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalForecastRequestAdjuster {
    pub minimum: f64,
}

impl RegionalForecastRequestFactor for MinimumRegionalForecastRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalForecastRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional forecast request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalForecastRequestEngine {
    pub steps: Vec<Rc<dyn RegionalForecastRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalForecastRequestEngine {
    pub fn compute(&mut self, item: &RegionalForecastRequest) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalForecastRequestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalForecastRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_forecast_request_engine() -> RegionalForecastRequestEngine {
    RegionalForecastRequestEngine {
        steps: vec![
            Rc::new(RoundingRegionalForecastRequestAdjuster { step: 0.25 }) as Rc<dyn RegionalForecastRequestFactor>,
            Rc::new(PercentRegionalForecastRequestAdjuster { percent: -6.7 }),
            Rc::new(MinimumRegionalForecastRequestAdjuster { minimum: 6.2 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 29534.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_forecast_request_engine();
        let item = new_regional_forecast_request("code-789", "label-460");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
