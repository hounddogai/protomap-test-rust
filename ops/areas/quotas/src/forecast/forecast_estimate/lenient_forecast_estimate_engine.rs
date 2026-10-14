use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient forecast estimate calculation.
pub trait LenientForecastEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientForecastEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientForecastEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientForecastEstimateAdjuster for TieredLenientForecastEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientForecastEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientForecastEstimateAdjuster {
    pub minimum: f64,
}

impl LenientForecastEstimateAdjuster for MinimumLenientForecastEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientForecastEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimateFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl LenientForecastEstimateAdjuster for LenientForecastEstimateFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &LenientForecastEstimate, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientForecastEstimateAdjuster {
    pub step: f64,
}

impl LenientForecastEstimateAdjuster for RoundingLenientForecastEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientForecastEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient forecast estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientForecastEstimateEngine {
    pub steps: Vec<Rc<dyn LenientForecastEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientForecastEstimateEngine {
    pub fn evaluate(&mut self, item: &LenientForecastEstimate) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientForecastEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientForecastEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_forecast_estimate_engine() -> LenientForecastEstimateEngine {
    LenientForecastEstimateEngine {
        steps: vec![
            Rc::new(TieredLenientForecastEstimateAdjuster {
                bounds: vec![16.0, 5756.0, 6505.0],
                rates: vec![0.88, 0.96, 1.0],
            }) as Rc<dyn LenientForecastEstimateAdjuster>,
            Rc::new(MinimumLenientForecastEstimateAdjuster { minimum: 6.2 }),
            Rc::new(LenientForecastEstimateFloorValueOffsetAdjuster { weight: 1.19 }),
            Rc::new(RoundingLenientForecastEstimateAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 97852.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_forecast_estimate_engine();
        let item = new_lenient_forecast_estimate("code-998", "label-329");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
