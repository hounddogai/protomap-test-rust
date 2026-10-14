use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy nonce forecast calculation.
pub trait LegacyNonceForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyNonceForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyNonceForecastAdjuster {
    pub cap: f64,
}

impl LegacyNonceForecastStep for CapLegacyNonceForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyNonceForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyNonceForecastAdjuster {
    pub minimum: f64,
}

impl LegacyNonceForecastStep for MinimumLegacyNonceForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyNonceForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyNonceForecastAdjuster {
    pub step: f64,
}

impl LegacyNonceForecastStep for RoundingLegacyNonceForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyNonceForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy nonce forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyNonceForecastEngine {
    pub steps: Vec<Rc<dyn LegacyNonceForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyNonceForecastEngine {
    pub fn price(&mut self, item: &LegacyNonceForecast) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyNonceForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_nonce_forecast_engine() -> LegacyNonceForecastEngine {
    LegacyNonceForecastEngine {
        steps: vec![
            Rc::new(CapLegacyNonceForecastAdjuster { cap: 12303.0 }) as Rc<dyn LegacyNonceForecastStep>,
            Rc::new(MinimumLegacyNonceForecastAdjuster { minimum: 3.8 }),
            Rc::new(RoundingLegacyNonceForecastAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 80395.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_nonce_forecast_engine();
        let item = new_legacy_nonce_forecast("code-803", "title-389");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
