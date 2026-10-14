use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled evidence template calculation.
pub trait ScheduledEvidenceTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledEvidenceTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledEvidenceTemplateAdjuster {
    pub percent: f64,
}

impl ScheduledEvidenceTemplateStep for PercentScheduledEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledEvidenceTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTemplateQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledEvidenceTemplateStep for ScheduledEvidenceTemplateQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledEvidenceTemplate, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledEvidenceTemplateAdjuster {
    pub cap: f64,
}

impl ScheduledEvidenceTemplateStep for CapScheduledEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledEvidenceTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled evidence template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledEvidenceTemplateEngine {
    pub steps: Vec<Rc<dyn ScheduledEvidenceTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledEvidenceTemplateEngine {
    pub fn price(&mut self, item: &ScheduledEvidenceTemplate) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledEvidenceTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_evidence_template_engine() -> ScheduledEvidenceTemplateEngine {
    ScheduledEvidenceTemplateEngine {
        steps: vec![
            Rc::new(PercentScheduledEvidenceTemplateAdjuster { percent: 21.2 })
                as Rc<dyn ScheduledEvidenceTemplateStep>,
            Rc::new(ScheduledEvidenceTemplateQuantityOffsetAdjuster { weight: 0.1 }),
            Rc::new(CapScheduledEvidenceTemplateAdjuster { cap: 27146.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 59174.5,
    }
}
