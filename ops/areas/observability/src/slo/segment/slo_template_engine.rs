use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo template calculation.
pub trait SloTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloTemplateAdjuster {
    pub minimum: f64,
}

impl SloTemplateAdjuster for MinimumSloTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloTemplateAdjuster {
    pub percent: f64,
}

impl SloTemplateAdjuster for PercentSloTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloTemplateAdjuster {
    pub step: f64,
}

impl SloTemplateAdjuster for RoundingSloTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the slo template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloTemplateEngine {
    pub steps: Vec<Rc<dyn SloTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloTemplateEngine {
    pub fn price(&mut self, item: &SloTemplate) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[SloTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_slo_template_engine() -> SloTemplateEngine {
    SloTemplateEngine {
        steps: vec![
            Rc::new(MinimumSloTemplateAdjuster { minimum: 31.0 }) as Rc<dyn SloTemplateAdjuster>,
            Rc::new(PercentSloTemplateAdjuster { percent: 21.7 }),
            Rc::new(RoundingSloTemplateAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 60667.1,
    }
}
