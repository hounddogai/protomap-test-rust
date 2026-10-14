use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast mapping calculation.
pub trait ForecastMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastMappingAdjuster {
    pub percent: f64,
}

impl ForecastMappingStep for PercentForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastMappingStep for TieredForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastMappingAdjuster {
    pub step: f64,
}

impl ForecastMappingStep for RoundingForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMappingWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ForecastMappingStep for ForecastMappingWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastMapping, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastMappingAdjuster {
    pub minimum: f64,
}

impl ForecastMappingStep for MinimumForecastMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the forecast mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastMappingEngine {
    pub steps: Vec<Rc<dyn ForecastMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastMappingEngine {
    pub fn price(&mut self, item: &ForecastMapping) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastMappingStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_forecast_mapping_engine() -> ForecastMappingEngine {
    ForecastMappingEngine {
        steps: vec![
            Rc::new(PercentForecastMappingAdjuster { percent: 10.2 }) as Rc<dyn ForecastMappingStep>,
            Rc::new(TieredForecastMappingAdjuster {
                bounds: vec![4930.0, 6674.0],
                rates: vec![1.07, 0.96],
            }),
            Rc::new(RoundingForecastMappingAdjuster { step: 5.0 }),
            Rc::new(ForecastMappingWindowDaysOffsetAdjuster { weight: 1.34 }),
            Rc::new(MinimumForecastMappingAdjuster { minimum: 14.7 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 29957.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_mapping_engine();
        let item = new_forecast_mapping("code-928", "title-614");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
