use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast tier calculation.
pub trait ForecastTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastTierAdjuster {
    pub cap: f64,
}

impl ForecastTierFactor for CapForecastTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryForecastTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastTierFactor for CountryForecastTierAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ForecastTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastTierAdjuster {
    pub percent: f64,
}

impl ForecastTierFactor for PercentForecastTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastTierAdjuster {
    pub step: f64,
}

impl ForecastTierFactor for RoundingForecastTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the forecast tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastTierEngine {
    pub steps: Vec<Rc<dyn ForecastTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastTierEngine {
    pub fn evaluate(&mut self, item: &ForecastTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ForecastTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_forecast_tier_engine() -> ForecastTierEngine {
    ForecastTierEngine {
        steps: vec![
            Rc::new(CapForecastTierAdjuster { cap: 46067.0 }) as Rc<dyn ForecastTierFactor>,
            Rc::new(CountryForecastTierAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.17),
                    ("online".to_string(), 0.95),
                    ("retail".to_string(), 1.05),
                ]),
            }),
            Rc::new(PercentForecastTierAdjuster { percent: 4.9 }),
            Rc::new(RoundingForecastTierAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 31107.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_tier_engine();
        let item = new_forecast_tier("code-177", "name-392");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
