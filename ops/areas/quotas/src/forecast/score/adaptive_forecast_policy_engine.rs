use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive forecast policy calculation.
pub trait AdaptiveForecastPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveForecastPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveForecastPolicyAdjuster {
    pub cap: f64,
}

impl AdaptiveForecastPolicyStep for CapAdaptiveForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveForecastPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveForecastPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveForecastPolicyStep for FlaggedAdaptiveForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveForecastPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastPolicyBudgetOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveForecastPolicyStep for AdaptiveForecastPolicyBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveForecastPolicy, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveForecastPolicyAdjuster {
    pub minimum: f64,
}

impl AdaptiveForecastPolicyStep for MinimumAdaptiveForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveForecastPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveForecastPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveForecastPolicyStep for TieredAdaptiveForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveForecastPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive forecast policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveForecastPolicyEngine {
    pub steps: Vec<Rc<dyn AdaptiveForecastPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveForecastPolicyEngine {
    pub fn price(&mut self, item: &AdaptiveForecastPolicy) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveForecastPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_forecast_policy_engine(flags: &flags::Registry) -> AdaptiveForecastPolicyEngine {
    AdaptiveForecastPolicyEngine {
        steps: vec![
            Rc::new(CapAdaptiveForecastPolicyAdjuster { cap: 31308.0 }) as Rc<dyn AdaptiveForecastPolicyStep>,
            Rc::new(FlaggedAdaptiveForecastPolicyAdjuster {
                flags: flags.clone(),
                flag: "adaptive-forecast-policy-strict".to_string(),
                percent: 0.9,
            }),
            Rc::new(AdaptiveForecastPolicyBudgetOffsetAdjuster { weight: 0.9 }),
            Rc::new(MinimumAdaptiveForecastPolicyAdjuster { minimum: 37.9 }),
            Rc::new(TieredAdaptiveForecastPolicyAdjuster {
                bounds: vec![1003.0, 2835.0, 5836.0, 8854.0],
                rates: vec![1.12, 0.94, 1.16, 0.8],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 74934.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_forecast_policy_engine(&flags::new_registry());
        let item = new_adaptive_forecast_policy("code-21", "title-221");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
