use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled ttl policy template calculation.
pub trait ScheduledTtlPolicyTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTtlPolicyTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledTtlPolicyTemplateAdjuster {
    pub percent: f64,
}

impl ScheduledTtlPolicyTemplateModifier for PercentScheduledTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledTtlPolicyTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyTemplateQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledTtlPolicyTemplateModifier for ScheduledTtlPolicyTemplateQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledTtlPolicyTemplate, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledTtlPolicyTemplateAdjuster {
    pub step: f64,
}

impl ScheduledTtlPolicyTemplateModifier for RoundingScheduledTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledTtlPolicyTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledTtlPolicyTemplateAdjuster {
    pub minimum: f64,
}

impl ScheduledTtlPolicyTemplateModifier for MinimumScheduledTtlPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledTtlPolicyTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled ttl policy template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTtlPolicyTemplateEngine {
    pub steps: Vec<Rc<dyn ScheduledTtlPolicyTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTtlPolicyTemplateEngine {
    pub fn run(&mut self, item: &ScheduledTtlPolicyTemplate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledTtlPolicyTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTtlPolicyTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_ttl_policy_template_engine() -> ScheduledTtlPolicyTemplateEngine {
    ScheduledTtlPolicyTemplateEngine {
        steps: vec![
            Rc::new(PercentScheduledTtlPolicyTemplateAdjuster { percent: 14.5 })
                as Rc<dyn ScheduledTtlPolicyTemplateModifier>,
            Rc::new(ScheduledTtlPolicyTemplateQuantityOffsetAdjuster { weight: 0.35 }),
            Rc::new(RoundingScheduledTtlPolicyTemplateAdjuster { step: 5.0 }),
            Rc::new(MinimumScheduledTtlPolicyTemplateAdjuster { minimum: 36.2 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 87142.7,
    }
}
