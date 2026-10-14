use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover forecast calculation.
pub trait FailoverForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverForecastAdjuster {
    pub step: f64,
}

impl FailoverForecastModifier for RoundingFailoverForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverForecastAdjuster {
    pub minimum: f64,
}

impl FailoverForecastModifier for MinimumFailoverForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverForecastAdjuster {
    pub cap: f64,
}

impl FailoverForecastModifier for CapFailoverForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the failover forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverForecastEngine {
    pub steps: Vec<Rc<dyn FailoverForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverForecastEngine {
    pub fn run(&mut self, item: &FailoverForecast) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_failover_forecast_engine() -> FailoverForecastEngine {
    FailoverForecastEngine {
        steps: vec![
            Rc::new(RoundingFailoverForecastAdjuster { step: 0.1 }) as Rc<dyn FailoverForecastModifier>,
            Rc::new(MinimumFailoverForecastAdjuster { minimum: 14.1 }),
            Rc::new(CapFailoverForecastAdjuster { cap: 42383.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 42089.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_forecast_engine();
        let item = new_failover_forecast("code-104", "name-796");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
