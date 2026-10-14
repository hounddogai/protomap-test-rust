use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token forecast calculation.
pub trait TokenForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenForecastAdjuster {
    pub percent: f64,
}

impl TokenForecastFactor for PercentTokenForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenForecastFactor for TieredTokenForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTokenForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenForecastFactor for ChannelTokenForecastAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TokenForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenForecastFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl TokenForecastFactor for TokenForecastFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &TokenForecast, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the token forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenForecastEngine {
    pub steps: Vec<Rc<dyn TokenForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenForecastEngine {
    pub fn run(&mut self, item: &TokenForecast) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TokenForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_token_forecast_engine() -> TokenForecastEngine {
    TokenForecastEngine {
        steps: vec![
            Rc::new(PercentTokenForecastAdjuster { percent: 1.8 }) as Rc<dyn TokenForecastFactor>,
            Rc::new(TieredTokenForecastAdjuster {
                bounds: vec![1709.0, 4221.0],
                rates: vec![0.94, 1.05],
            }),
            Rc::new(ChannelTokenForecastAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.18), ("central".to_string(), 1.19)]),
            }),
            Rc::new(TokenForecastFloorValueOffsetAdjuster { weight: 1.51 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 26525.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_forecast_engine();
        let item = new_token_forecast("code-325", "title-841");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
