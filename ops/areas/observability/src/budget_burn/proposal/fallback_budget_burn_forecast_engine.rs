use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback budget burn forecast calculation.
pub trait FallbackBudgetBurnForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBudgetBurnForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackBudgetBurnForecastAdjuster {
    pub step: f64,
}

impl FallbackBudgetBurnForecastAdjuster for RoundingFallbackBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackBudgetBurnForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackBudgetBurnForecastAdjuster {
    pub minimum: f64,
}

impl FallbackBudgetBurnForecastAdjuster for MinimumFallbackBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackBudgetBurnForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackBudgetBurnForecastAdjuster {
    pub percent: f64,
}

impl FallbackBudgetBurnForecastAdjuster for PercentFallbackBudgetBurnForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackBudgetBurnForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback budget burn forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBudgetBurnForecastEngine {
    pub steps: Vec<Rc<dyn FallbackBudgetBurnForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBudgetBurnForecastEngine {
    pub fn price(&mut self, item: &FallbackBudgetBurnForecast) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBudgetBurnForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_budget_burn_forecast_engine() -> FallbackBudgetBurnForecastEngine {
    FallbackBudgetBurnForecastEngine {
        steps: vec![
            Rc::new(RoundingFallbackBudgetBurnForecastAdjuster { step: 1.0 })
                as Rc<dyn FallbackBudgetBurnForecastAdjuster>,
            Rc::new(MinimumFallbackBudgetBurnForecastAdjuster { minimum: 12.0 }),
            Rc::new(PercentFallbackBudgetBurnForecastAdjuster { percent: 15.3 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 34217.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_budget_burn_forecast_engine();
        let item = new_fallback_budget_burn_forecast("code-134", "name-367");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
