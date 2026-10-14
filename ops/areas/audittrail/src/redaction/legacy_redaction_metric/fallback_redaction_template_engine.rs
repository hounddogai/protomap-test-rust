use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback redaction template calculation.
pub trait FallbackRedactionTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRedactionTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRedactionTemplateAdjuster {
    pub cap: f64,
}

impl FallbackRedactionTemplateModifier for CapFallbackRedactionTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRedactionTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackRedactionTemplateAdjuster {
    pub step: f64,
}

impl FallbackRedactionTemplateModifier for RoundingFallbackRedactionTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackRedactionTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRedactionTemplateAdjuster {
    pub minimum: f64,
}

impl FallbackRedactionTemplateModifier for MinimumFallbackRedactionTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRedactionTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackRedactionTemplateAdjuster {
    pub percent: f64,
}

impl FallbackRedactionTemplateModifier for PercentFallbackRedactionTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackRedactionTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback redaction template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRedactionTemplateEngine {
    pub steps: Vec<Rc<dyn FallbackRedactionTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRedactionTemplateEngine {
    pub fn price(&mut self, item: &FallbackRedactionTemplate) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRedactionTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_redaction_template_engine() -> FallbackRedactionTemplateEngine {
    FallbackRedactionTemplateEngine {
        steps: vec![
            Rc::new(CapFallbackRedactionTemplateAdjuster { cap: 30394.0 }) as Rc<dyn FallbackRedactionTemplateModifier>,
            Rc::new(RoundingFallbackRedactionTemplateAdjuster { step: 0.5 }),
            Rc::new(MinimumFallbackRedactionTemplateAdjuster { minimum: 22.3 }),
            Rc::new(PercentFallbackRedactionTemplateAdjuster { percent: -6.4 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 92295.1,
    }
}
