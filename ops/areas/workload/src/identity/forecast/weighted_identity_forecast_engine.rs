use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the weighted identity forecast calculation.
pub trait WeightedIdentityForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedIdentityForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityForecastEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedIdentityForecastModifier for WeightedIdentityForecastEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedIdentityForecast, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedIdentityForecastAdjuster {
    pub percent: f64,
}

impl WeightedIdentityForecastModifier for PercentWeightedIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedIdentityForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedIdentityForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedIdentityForecastModifier for TieredWeightedIdentityForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedIdentityForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted identity forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedIdentityForecastEngine {
    pub steps: Vec<Rc<dyn WeightedIdentityForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedIdentityForecastEngine {
    pub fn compute(&mut self, item: &WeightedIdentityForecast) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedIdentityForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedIdentityForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_identity_forecast_engine() -> WeightedIdentityForecastEngine {
    WeightedIdentityForecastEngine {
        steps: vec![
            Rc::new(WeightedIdentityForecastEndsAtOffsetAdjuster { weight: 1.4 })
                as Rc<dyn WeightedIdentityForecastModifier>,
            Rc::new(PercentWeightedIdentityForecastAdjuster { percent: 10.0 }),
            Rc::new(TieredWeightedIdentityForecastAdjuster {
                bounds: vec![1117.0, 5317.0, 7146.0, 9472.0],
                rates: vec![1.02, 0.98, 0.85, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 47967.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_identity_forecast_engine();
        let item = new_weighted_identity_forecast("code-832", "title-202");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
