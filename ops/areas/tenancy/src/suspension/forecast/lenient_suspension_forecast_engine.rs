use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient suspension forecast calculation.
pub trait LenientSuspensionForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSuspensionForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientSuspensionForecastAdjuster {
    pub percent: f64,
}

impl LenientSuspensionForecastFactor for PercentLenientSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientSuspensionForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientSuspensionForecastAdjuster {
    pub step: f64,
}

impl LenientSuspensionForecastFactor for RoundingLenientSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientSuspensionForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSuspensionForecastAdjuster {
    pub cap: f64,
}

impl LenientSuspensionForecastFactor for CapLenientSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSuspensionForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionForecastExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl LenientSuspensionForecastFactor for LenientSuspensionForecastExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &LenientSuspensionForecast, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLenientSuspensionForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientSuspensionForecastFactor for SourceLenientSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LenientSuspensionForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSuspensionForecastAdjuster {
    pub minimum: f64,
}

impl LenientSuspensionForecastFactor for MinimumLenientSuspensionForecastAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSuspensionForecast, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient suspension forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSuspensionForecastEngine {
    pub steps: Vec<Rc<dyn LenientSuspensionForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSuspensionForecastEngine {
    pub fn price(&mut self, item: &LenientSuspensionForecast) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientSuspensionForecastFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientSuspensionForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_suspension_forecast_engine() -> LenientSuspensionForecastEngine {
    LenientSuspensionForecastEngine {
        steps: vec![
            Rc::new(PercentLenientSuspensionForecastAdjuster { percent: -11.8 })
                as Rc<dyn LenientSuspensionForecastFactor>,
            Rc::new(RoundingLenientSuspensionForecastAdjuster { step: 0.5 }),
            Rc::new(CapLenientSuspensionForecastAdjuster { cap: 35619.0 }),
            Rc::new(LenientSuspensionForecastExpiresAtOffsetAdjuster { weight: 1.84 }),
            Rc::new(SourceLenientSuspensionForecastAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.1),
                    ("east".to_string(), 1.0),
                    ("north".to_string(), 1.03),
                ]),
            }),
            Rc::new(MinimumLenientSuspensionForecastAdjuster { minimum: 2.3 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 11254.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_suspension_forecast_engine();
        let item = new_lenient_suspension_forecast("code-837", "label-615");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
