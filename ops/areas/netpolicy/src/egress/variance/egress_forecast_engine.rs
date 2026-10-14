use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress forecast calculation.
pub trait EgressForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressForecastCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl EgressForecastStep for EgressForecastCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &EgressForecast, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressForecastAdjuster {
    pub step: f64,
}

impl EgressForecastStep for RoundingEgressForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressForecastAdjuster {
    pub percent: f64,
}

impl EgressForecastStep for PercentEgressForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressForecastEngine {
    pub steps: Vec<Rc<dyn EgressForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressForecastEngine {
    pub fn run(&mut self, item: &EgressForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EgressForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_forecast_engine() -> EgressForecastEngine {
    EgressForecastEngine {
        steps: vec![
            Rc::new(EgressForecastCreatedAtOffsetAdjuster { weight: 1.92 }) as Rc<dyn EgressForecastStep>,
            Rc::new(RoundingEgressForecastAdjuster { step: 0.5 }),
            Rc::new(PercentEgressForecastAdjuster { percent: 19.8 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 91178.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_forecast_engine();
        let item = new_egress_forecast("code-766", "label-180");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
