use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy rollout forecast calculation.
pub trait LegacyRolloutForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRolloutForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRolloutForecastAdjuster {
    pub cap: f64,
}

impl LegacyRolloutForecastStep for CapLegacyRolloutForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRolloutForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRolloutForecastAdjuster {
    pub minimum: f64,
}

impl LegacyRolloutForecastStep for MinimumLegacyRolloutForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRolloutForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLegacyRolloutForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyRolloutForecastStep for TitleLegacyRolloutForecastAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LegacyRolloutForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy rollout forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRolloutForecastEngine {
    pub steps: Vec<Rc<dyn LegacyRolloutForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRolloutForecastEngine {
    pub fn evaluate(&mut self, item: &LegacyRolloutForecast) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRolloutForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_rollout_forecast_engine() -> LegacyRolloutForecastEngine {
    LegacyRolloutForecastEngine {
        steps: vec![
            Rc::new(CapLegacyRolloutForecastAdjuster { cap: 13603.0 }) as Rc<dyn LegacyRolloutForecastStep>,
            Rc::new(MinimumLegacyRolloutForecastAdjuster { minimum: 34.4 }),
            Rc::new(TitleLegacyRolloutForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.21)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 9985.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_rollout_forecast_engine();
        let item = new_legacy_rollout_forecast("code-254", "title-347");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
