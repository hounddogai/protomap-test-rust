use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback release exception calculation.
pub trait FallbackReleaseExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackReleaseException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackReleaseExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackReleaseExceptionModifier for FlaggedFallbackReleaseExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackReleaseException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackReleaseExceptionAdjuster {
    pub minimum: f64,
}

impl FallbackReleaseExceptionModifier for MinimumFallbackReleaseExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackReleaseException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackReleaseExceptionAdjuster {
    pub cap: f64,
}

impl FallbackReleaseExceptionModifier for CapFallbackReleaseExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackReleaseException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback release exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackReleaseExceptionEngine {
    pub steps: Vec<Rc<dyn FallbackReleaseExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackReleaseExceptionEngine {
    pub fn price(&mut self, item: &FallbackReleaseException) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackReleaseException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_release_exception_engine(flags: &flags::Registry) -> FallbackReleaseExceptionEngine {
    FallbackReleaseExceptionEngine {
        steps: vec![
            Rc::new(FlaggedFallbackReleaseExceptionAdjuster {
                flags: flags.clone(),
                flag: "fallback-release-exception-regional".to_string(),
                percent: 7.9,
            }) as Rc<dyn FallbackReleaseExceptionModifier>,
            Rc::new(MinimumFallbackReleaseExceptionAdjuster { minimum: 33.7 }),
            Rc::new(CapFallbackReleaseExceptionAdjuster { cap: 40744.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 94503.4,
    }
}
