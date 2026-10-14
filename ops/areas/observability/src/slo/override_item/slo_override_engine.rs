use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the slo override calculation.
pub trait SloOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloOverrideAdjuster {
    pub percent: f64,
}

impl SloOverrideFactor for PercentSloOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloOverrideAdjuster {
    pub cap: f64,
}

impl SloOverrideFactor for CapSloOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloOverrideWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl SloOverrideFactor for SloOverrideWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &SloOverride, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloOverrideAdjuster {
    pub minimum: f64,
}

impl SloOverrideFactor for MinimumSloOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the slo override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloOverrideEngine {
    pub steps: Vec<Rc<dyn SloOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloOverrideEngine {
    pub fn price(&mut self, item: &SloOverride) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloOverrideFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_slo_override_engine() -> SloOverrideEngine {
    SloOverrideEngine {
        steps: vec![
            Rc::new(PercentSloOverrideAdjuster { percent: -14.7 }) as Rc<dyn SloOverrideFactor>,
            Rc::new(CapSloOverrideAdjuster { cap: 46865.0 }),
            Rc::new(SloOverrideWindowDaysOffsetAdjuster { weight: 0.51 }),
            Rc::new(MinimumSloOverrideAdjuster { minimum: 28.8 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 8570.1,
    }
}
