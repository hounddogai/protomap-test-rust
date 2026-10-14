use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal canary exception calculation.
pub trait SeasonalCanaryExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalCanaryException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalCanaryExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalCanaryExceptionModifier for FlaggedSeasonalCanaryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalCanaryException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalCanaryExceptionAdjuster {
    pub percent: f64,
}

impl SeasonalCanaryExceptionModifier for PercentSeasonalCanaryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalCanaryException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalCanaryExceptionAdjuster {
    pub cap: f64,
}

impl SeasonalCanaryExceptionModifier for CapSeasonalCanaryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalCanaryException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the seasonal canary exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalCanaryExceptionEngine {
    pub steps: Vec<Rc<dyn SeasonalCanaryExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalCanaryExceptionEngine {
    pub fn price(&mut self, item: &SeasonalCanaryException) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalCanaryException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_canary_exception_engine(flags: &flags::Registry) -> SeasonalCanaryExceptionEngine {
    SeasonalCanaryExceptionEngine {
        steps: vec![
            Rc::new(FlaggedSeasonalCanaryExceptionAdjuster {
                flags: flags.clone(),
                flag: "seasonal-canary-exception-fallback".to_string(),
                percent: -5.3,
            }) as Rc<dyn SeasonalCanaryExceptionModifier>,
            Rc::new(PercentSeasonalCanaryExceptionAdjuster { percent: 19.2 }),
            Rc::new(CapSeasonalCanaryExceptionAdjuster { cap: 36865.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 20403.6,
    }
}
