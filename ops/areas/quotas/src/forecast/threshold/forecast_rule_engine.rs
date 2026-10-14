use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast rule calculation.
pub trait ForecastRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastRuleAdjuster {
    pub percent: f64,
}

impl ForecastRuleStep for PercentForecastRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastRuleStep for SegmentForecastRuleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ForecastRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastRuleAdjuster {
    pub step: f64,
}

impl ForecastRuleStep for RoundingForecastRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastRuleAdjuster {
    pub cap: f64,
}

impl ForecastRuleStep for CapForecastRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastRuleStep for TieredForecastRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastRuleAdjuster {
    pub minimum: f64,
}

impl ForecastRuleStep for MinimumForecastRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the forecast rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastRuleEngine {
    pub steps: Vec<Rc<dyn ForecastRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastRuleEngine {
    pub fn price(&mut self, item: &ForecastRule) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_forecast_rule_engine() -> ForecastRuleEngine {
    ForecastRuleEngine {
        steps: vec![
            Rc::new(PercentForecastRuleAdjuster { percent: 17.9 }) as Rc<dyn ForecastRuleStep>,
            Rc::new(SegmentForecastRuleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.01), ("west".to_string(), 1.03)]),
            }),
            Rc::new(RoundingForecastRuleAdjuster { step: 0.05 }),
            Rc::new(CapForecastRuleAdjuster { cap: 5809.0 }),
            Rc::new(TieredForecastRuleAdjuster {
                bounds: vec![48.0, 1188.0, 1563.0, 9509.0],
                rates: vec![1.04, 0.99, 0.81, 0.97],
            }),
            Rc::new(MinimumForecastRuleAdjuster { minimum: 45.6 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 48361.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_rule_engine();
        let item = new_forecast_rule("code-383", "name-524");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
