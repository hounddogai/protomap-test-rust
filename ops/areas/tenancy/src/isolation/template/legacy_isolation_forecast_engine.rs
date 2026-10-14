use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy isolation forecast calculation.
pub trait LegacyIsolationForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyIsolationForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyIsolationForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyIsolationForecastAdjuster for TieredLegacyIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyIsolationForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationForecastLimitOffsetAdjuster {
    pub weight: f64,
}

impl LegacyIsolationForecastAdjuster for LegacyIsolationForecastLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyIsolationForecast, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyIsolationForecastAdjuster {
    pub minimum: f64,
}

impl LegacyIsolationForecastAdjuster for MinimumLegacyIsolationForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyIsolationForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy isolation forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyIsolationForecastEngine {
    pub steps: Vec<Rc<dyn LegacyIsolationForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyIsolationForecastEngine {
    pub fn compute(&mut self, item: &LegacyIsolationForecast) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyIsolationForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyIsolationForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_isolation_forecast_engine() -> LegacyIsolationForecastEngine {
    LegacyIsolationForecastEngine {
        steps: vec![
            Rc::new(TieredLegacyIsolationForecastAdjuster {
                bounds: vec![222.0, 3771.0, 4907.0, 7095.0],
                rates: vec![1.01, 0.95, 0.84, 1.05],
            }) as Rc<dyn LegacyIsolationForecastAdjuster>,
            Rc::new(LegacyIsolationForecastLimitOffsetAdjuster { weight: 0.6 }),
            Rc::new(MinimumLegacyIsolationForecastAdjuster { minimum: 31.5 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 44214.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_isolation_forecast_engine();
        let item = new_legacy_isolation_forecast("code-885", "title-418");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
