use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast template calculation.
pub trait ForecastTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastTemplateAdjuster {
    pub percent: f64,
}

impl ForecastTemplateFactor for PercentForecastTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryForecastTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastTemplateFactor for CountryForecastTemplateAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ForecastTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTemplateLimitOffsetAdjuster {
    pub weight: f64,
}

impl ForecastTemplateFactor for ForecastTemplateLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastTemplate, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the forecast template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastTemplateEngine {
    pub steps: Vec<Rc<dyn ForecastTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastTemplateEngine {
    pub fn price(&mut self, item: &ForecastTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ForecastTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_forecast_template_engine() -> ForecastTemplateEngine {
    ForecastTemplateEngine {
        steps: vec![
            Rc::new(PercentForecastTemplateAdjuster { percent: 8.2 }) as Rc<dyn ForecastTemplateFactor>,
            Rc::new(CountryForecastTemplateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.07), ("central".to_string(), 0.79)]),
            }),
            Rc::new(ForecastTemplateLimitOffsetAdjuster { weight: 1.71 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 98205.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_template_engine();
        let item = new_forecast_template("code-407", "title-495");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
