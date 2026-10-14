use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rule forecast calculation.
pub trait RuleForecastAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleForecastAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleForecastAdjuster for TieredRuleForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleForecast, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleForecastAdjuster for FlaggedRuleForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleForecastAdjuster {
    pub percent: f64,
}

impl RuleForecastAdjuster for PercentRuleForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rule forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleForecastEngine {
    pub steps: Vec<Rc<dyn RuleForecastAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleForecastEngine {
    pub fn evaluate(&mut self, item: &RuleForecast) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleForecastAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rule_forecast_engine(flags: &flags::Registry) -> RuleForecastEngine {
    RuleForecastEngine {
        steps: vec![
            Rc::new(TieredRuleForecastAdjuster {
                bounds: vec![271.0, 3507.0],
                rates: vec![1.07, 0.87],
            }) as Rc<dyn RuleForecastAdjuster>,
            Rc::new(FlaggedRuleForecastAdjuster {
                flags: flags.clone(),
                flag: "rule-forecast-weighted".to_string(),
                percent: -7.4,
            }),
            Rc::new(PercentRuleForecastAdjuster { percent: -11.7 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 96228.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_forecast_engine(&flags::new_registry());
        let item = new_rule_forecast("code-449", "label-834");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
