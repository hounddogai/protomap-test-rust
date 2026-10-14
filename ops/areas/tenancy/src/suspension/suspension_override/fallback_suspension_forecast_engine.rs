use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback suspension forecast calculation.
pub trait FallbackSuspensionForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSuspensionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionForecastExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackSuspensionForecastAdjuster for FallbackSuspensionForecastExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackSuspensionForecast, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackSuspensionForecastAdjuster {
    pub minimum: f64,
}

impl FallbackSuspensionForecastAdjuster for MinimumFallbackSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackSuspensionForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackSuspensionForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackSuspensionForecastAdjuster for TieredFallbackSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackSuspensionForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback suspension forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSuspensionForecastEngine {
    pub steps: Vec<Rc<dyn FallbackSuspensionForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSuspensionForecastEngine {
    pub fn run(&mut self, item: &FallbackSuspensionForecast) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackSuspensionForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSuspensionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_suspension_forecast_engine() -> FallbackSuspensionForecastEngine {
    FallbackSuspensionForecastEngine {
        steps: vec![
            Rc::new(FallbackSuspensionForecastExpiresAtOffsetAdjuster { weight: 0.31 })
                as Rc<dyn FallbackSuspensionForecastAdjuster>,
            Rc::new(MinimumFallbackSuspensionForecastAdjuster { minimum: 12.9 }),
            Rc::new(TieredFallbackSuspensionForecastAdjuster {
                bounds: vec![1199.0, 2900.0, 5732.0, 7315.0],
                rates: vec![1.07, 1.02, 1.03, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 13124.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_suspension_forecast_engine();
        let item = new_fallback_suspension_forecast("code-485", "name-454");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
