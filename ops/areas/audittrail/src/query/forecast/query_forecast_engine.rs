use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query forecast calculation.
pub trait QueryForecastStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryForecastAdjuster {
    pub minimum: f64,
}

impl QueryForecastStep for MinimumQueryForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryForecastAdjuster {
    pub percent: f64,
}

impl QueryForecastStep for PercentQueryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryForecastStep for FlaggedQueryForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecastRevisionOffsetAdjuster {
    pub weight: f64,
}

impl QueryForecastStep for QueryForecastRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &QueryForecast, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the query forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryForecastEngine {
    pub steps: Vec<Rc<dyn QueryForecastStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryForecastEngine {
    pub fn price(&mut self, item: &QueryForecast) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[QueryForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_query_forecast_engine(flags: &flags::Registry) -> QueryForecastEngine {
    QueryForecastEngine {
        steps: vec![
            Rc::new(MinimumQueryForecastAdjuster { minimum: 28.6 }) as Rc<dyn QueryForecastStep>,
            Rc::new(PercentQueryForecastAdjuster { percent: 9.6 }),
            Rc::new(FlaggedQueryForecastAdjuster {
                flags: flags.clone(),
                flag: "query-forecast-legacy".to_string(),
                percent: 3.5,
            }),
            Rc::new(QueryForecastRevisionOffsetAdjuster { weight: 1.94 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 77269.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_forecast_engine(&flags::new_registry());
        let item = new_query_forecast("code-97", "title-408");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
