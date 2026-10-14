use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast variance calculation.
pub trait ForecastVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastVarianceAdjuster {
    pub cap: f64,
}

impl ForecastVarianceFactor for CapForecastVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastVarianceFactor for TieredForecastVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastVarianceCostOffsetAdjuster {
    pub weight: f64,
}

impl ForecastVarianceFactor for ForecastVarianceCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastVariance, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleForecastVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastVarianceFactor for TitleForecastVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ForecastVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the forecast variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastVarianceEngine {
    pub steps: Vec<Rc<dyn ForecastVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastVarianceEngine {
    pub fn compute(&mut self, item: &ForecastVariance) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ForecastVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_forecast_variance_engine() -> ForecastVarianceEngine {
    ForecastVarianceEngine {
        steps: vec![
            Rc::new(CapForecastVarianceAdjuster { cap: 35969.0 }) as Rc<dyn ForecastVarianceFactor>,
            Rc::new(TieredForecastVarianceAdjuster {
                bounds: vec![412.0, 630.0, 2474.0, 9961.0],
                rates: vec![1.1, 1.11, 1.16, 0.89],
            }),
            Rc::new(ForecastVarianceCostOffsetAdjuster { weight: 1.03 }),
            Rc::new(TitleForecastVarianceAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.09),
                    ("retail".to_string(), 1.31),
                    ("west".to_string(), 0.85),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 80867.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_variance_engine();
        let item = new_forecast_variance("code-883", "title-241");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
