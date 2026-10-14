use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient silence override calculation.
pub trait LenientSilenceOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSilenceOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientSilenceOverrideAdjuster {
    pub percent: f64,
}

impl LenientSilenceOverrideModifier for PercentLenientSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientSilenceOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientSilenceOverrideAdjuster {
    pub step: f64,
}

impl LenientSilenceOverrideModifier for RoundingLenientSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientSilenceOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSilenceOverrideAdjuster {
    pub cap: f64,
}

impl LenientSilenceOverrideModifier for CapLenientSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSilenceOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient silence override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSilenceOverrideEngine {
    pub steps: Vec<Rc<dyn LenientSilenceOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSilenceOverrideEngine {
    pub fn run(&mut self, item: &LenientSilenceOverride) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[LenientSilenceOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_silence_override_engine() -> LenientSilenceOverrideEngine {
    LenientSilenceOverrideEngine {
        steps: vec![
            Rc::new(PercentLenientSilenceOverrideAdjuster { percent: 25.0 }) as Rc<dyn LenientSilenceOverrideModifier>,
            Rc::new(RoundingLenientSilenceOverrideAdjuster { step: 1.0 }),
            Rc::new(CapLenientSilenceOverrideAdjuster { cap: 41253.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 8155.2,
    }
}
