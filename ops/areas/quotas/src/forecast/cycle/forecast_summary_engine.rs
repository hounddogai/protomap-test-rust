use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast summary calculation.
pub trait ForecastSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastSummaryAdjuster {
    pub step: f64,
}

impl ForecastSummaryModifier for RoundingForecastSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastSummaryModifier for TieredForecastSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryForecastSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastSummaryModifier for CountryForecastSummaryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ForecastSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastSummaryAdjuster {
    pub minimum: f64,
}

impl ForecastSummaryModifier for MinimumForecastSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the forecast summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastSummaryEngine {
    pub steps: Vec<Rc<dyn ForecastSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastSummaryEngine {
    pub fn evaluate(&mut self, item: &ForecastSummary) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[ForecastSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_forecast_summary_engine() -> ForecastSummaryEngine {
    ForecastSummaryEngine {
        steps: vec![
            Rc::new(RoundingForecastSummaryAdjuster { step: 5.0 }) as Rc<dyn ForecastSummaryModifier>,
            Rc::new(TieredForecastSummaryAdjuster {
                bounds: vec![2004.0, 2544.0, 4418.0, 7470.0],
                rates: vec![0.89, 1.01, 1.1, 0.95],
            }),
            Rc::new(CountryForecastSummaryAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.92)]),
            }),
            Rc::new(MinimumForecastSummaryAdjuster { minimum: 3.5 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 10718.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_summary_engine();
        let item = new_forecast_summary("code-604", "title-886");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
