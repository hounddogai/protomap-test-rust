use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient usage forecast calculation.
pub trait LenientUsageForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientUsageForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientUsageForecastAdjuster {
    pub cap: f64,
}

impl LenientUsageForecastFactor for CapLenientUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientUsageForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientUsageForecastAdjuster {
    pub minimum: f64,
}

impl LenientUsageForecastFactor for MinimumLenientUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientUsageForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientUsageForecastAdjuster {
    pub percent: f64,
}

impl LenientUsageForecastFactor for PercentLenientUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientUsageForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientUsageForecastAdjuster {
    pub step: f64,
}

impl LenientUsageForecastFactor for RoundingLenientUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientUsageForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientUsageForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientUsageForecastFactor for TieredLenientUsageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientUsageForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecastDiscountOffsetAdjuster {
    pub weight: f64,
}

impl LenientUsageForecastFactor for LenientUsageForecastDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &LenientUsageForecast, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the lenient usage forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientUsageForecastEngine {
    pub steps: Vec<Rc<dyn LenientUsageForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientUsageForecastEngine {
    pub fn price(&mut self, item: &LenientUsageForecast) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientUsageForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientUsageForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_usage_forecast_engine() -> LenientUsageForecastEngine {
    LenientUsageForecastEngine {
        steps: vec![
            Rc::new(CapLenientUsageForecastAdjuster { cap: 29523.0 }) as Rc<dyn LenientUsageForecastFactor>,
            Rc::new(MinimumLenientUsageForecastAdjuster { minimum: 45.1 }),
            Rc::new(PercentLenientUsageForecastAdjuster { percent: -14.1 }),
            Rc::new(RoundingLenientUsageForecastAdjuster { step: 1.0 }),
            Rc::new(TieredLenientUsageForecastAdjuster {
                bounds: vec![4868.0, 4975.0],
                rates: vec![1.06, 1.03],
            }),
            Rc::new(LenientUsageForecastDiscountOffsetAdjuster { weight: 1.41 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 45247.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_usage_forecast_engine();
        let item = new_lenient_usage_forecast("code-316", "name-581");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
