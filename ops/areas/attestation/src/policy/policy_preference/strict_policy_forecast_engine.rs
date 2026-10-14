use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict policy forecast calculation.
pub trait StrictPolicyForecastFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPolicyForecast, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictPolicyForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictPolicyForecastFactor for LabelStrictPolicyForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictPolicyForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictPolicyForecastAdjuster {
    pub percent: f64,
}

impl StrictPolicyForecastFactor for PercentStrictPolicyForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictPolicyForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictPolicyForecastAdjuster {
    pub cap: f64,
}

impl StrictPolicyForecastFactor for CapStrictPolicyForecastAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictPolicyForecast, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictPolicyForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictPolicyForecastFactor for FlaggedStrictPolicyForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictPolicyForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictPolicyForecastAdjuster {
    pub step: f64,
}

impl StrictPolicyForecastFactor for RoundingStrictPolicyForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictPolicyForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the strict policy forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPolicyForecastEngine {
    pub steps: Vec<Rc<dyn StrictPolicyForecastFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPolicyForecastEngine {
    pub fn compute(&mut self, item: &StrictPolicyForecast) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[StrictPolicyForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_policy_forecast_engine(flags: &flags::Registry) -> StrictPolicyForecastEngine {
    StrictPolicyForecastEngine {
        steps: vec![
            Rc::new(LabelStrictPolicyForecastAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.37), ("south".to_string(), 0.92)]),
            }) as Rc<dyn StrictPolicyForecastFactor>,
            Rc::new(PercentStrictPolicyForecastAdjuster { percent: 11.6 }),
            Rc::new(CapStrictPolicyForecastAdjuster { cap: 17254.0 }),
            Rc::new(FlaggedStrictPolicyForecastAdjuster {
                flags: flags.clone(),
                flag: "strict-policy-forecast-legacy".to_string(),
                percent: -0.7,
            }),
            Rc::new(RoundingStrictPolicyForecastAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 94792.4,
    }
}
