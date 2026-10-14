use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback page adjustment calculation.
pub trait FallbackPageAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPageAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackPageAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackPageAdjustmentModifier for FlaggedFallbackPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackPageAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPageAdjustmentAdjuster {
    pub minimum: f64,
}

impl FallbackPageAdjustmentModifier for MinimumFallbackPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPageAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPageAdjustmentAdjuster {
    pub step: f64,
}

impl FallbackPageAdjustmentModifier for RoundingFallbackPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPageAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the fallback page adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPageAdjustmentEngine {
    pub steps: Vec<Rc<dyn FallbackPageAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPageAdjustmentEngine {
    pub fn compute(&mut self, item: &FallbackPageAdjustment) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPageAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_page_adjustment_engine(flags: &flags::Registry) -> FallbackPageAdjustmentEngine {
    FallbackPageAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedFallbackPageAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "fallback-page-adjustment-primary".to_string(),
                percent: 0.5,
            }) as Rc<dyn FallbackPageAdjustmentModifier>,
            Rc::new(MinimumFallbackPageAdjustmentAdjuster { minimum: 46.5 }),
            Rc::new(RoundingFallbackPageAdjustmentAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 73169.5,
    }
}
