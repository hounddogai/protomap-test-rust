use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global forecast baseline calculation.
pub trait GlobalForecastBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalForecastBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalForecastBaselineAdjuster {
    pub step: f64,
}

impl GlobalForecastBaselineAdjuster for RoundingGlobalForecastBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalForecastBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalForecastBaselineAdjuster {
    pub percent: f64,
}

impl GlobalForecastBaselineAdjuster for PercentGlobalForecastBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalForecastBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalForecastBaselineAdjuster {
    pub cap: f64,
}

impl GlobalForecastBaselineAdjuster for CapGlobalForecastBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalForecastBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalForecastBaselineAdjuster {
    pub minimum: f64,
}

impl GlobalForecastBaselineAdjuster for MinimumGlobalForecastBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalForecastBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalForecastBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalForecastBaselineAdjuster for FlaggedGlobalForecastBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalForecastBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastBaselineRevisionOffsetAdjuster {
    pub weight: f64,
}

impl GlobalForecastBaselineAdjuster for GlobalForecastBaselineRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalForecastBaseline, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the global forecast baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalForecastBaselineEngine {
    pub steps: Vec<Rc<dyn GlobalForecastBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalForecastBaselineEngine {
    pub fn price(&mut self, item: &GlobalForecastBaseline) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalForecastBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_forecast_baseline_engine(flags: &flags::Registry) -> GlobalForecastBaselineEngine {
    GlobalForecastBaselineEngine {
        steps: vec![
            Rc::new(RoundingGlobalForecastBaselineAdjuster { step: 5.0 }) as Rc<dyn GlobalForecastBaselineAdjuster>,
            Rc::new(PercentGlobalForecastBaselineAdjuster { percent: -23.5 }),
            Rc::new(CapGlobalForecastBaselineAdjuster { cap: 38212.0 }),
            Rc::new(MinimumGlobalForecastBaselineAdjuster { minimum: 7.1 }),
            Rc::new(FlaggedGlobalForecastBaselineAdjuster {
                flags: flags.clone(),
                flag: "global-forecast-baseline-default".to_string(),
                percent: -7.9,
            }),
            Rc::new(GlobalForecastBaselineRevisionOffsetAdjuster { weight: 1.14 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 86938.4,
    }
}
