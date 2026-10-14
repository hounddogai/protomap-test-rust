use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending plan template calculation.
pub trait PendingPlanTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPlanTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingPlanTemplateAdjuster {
    pub step: f64,
}

impl PendingPlanTemplateModifier for RoundingPendingPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingPlanTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingPlanTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPlanTemplateModifier for NamePendingPlanTemplateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingPlanTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingPlanTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingPlanTemplateModifier for TieredPendingPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingPlanTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPlanTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPlanTemplateModifier for FlaggedPendingPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPlanTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingPlanTemplateAdjuster {
    pub percent: f64,
}

impl PendingPlanTemplateModifier for PercentPendingPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingPlanTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending plan template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPlanTemplateEngine {
    pub steps: Vec<Rc<dyn PendingPlanTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPlanTemplateEngine {
    pub fn evaluate(&mut self, item: &PendingPlanTemplate) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingPlanTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_plan_template_engine(flags: &flags::Registry) -> PendingPlanTemplateEngine {
    PendingPlanTemplateEngine {
        steps: vec![
            Rc::new(RoundingPendingPlanTemplateAdjuster { step: 1.0 }) as Rc<dyn PendingPlanTemplateModifier>,
            Rc::new(NamePendingPlanTemplateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.09), ("central".to_string(), 1.03)]),
            }),
            Rc::new(TieredPendingPlanTemplateAdjuster {
                bounds: vec![3379.0, 8923.0],
                rates: vec![0.99, 0.83],
            }),
            Rc::new(FlaggedPendingPlanTemplateAdjuster {
                flags: flags.clone(),
                flag: "pending-plan-template-rolling".to_string(),
                percent: 9.2,
            }),
            Rc::new(PercentPendingPlanTemplateAdjuster { percent: -12.3 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 18991.4,
    }
}
