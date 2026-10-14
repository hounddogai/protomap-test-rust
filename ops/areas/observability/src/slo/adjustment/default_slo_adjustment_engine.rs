use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default slo adjustment calculation.
pub trait DefaultSloAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultSloAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultSloAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultSloAdjustmentModifier for FlaggedDefaultSloAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultSloAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultSloAdjustmentAdjuster {
    pub percent: f64,
}

impl DefaultSloAdjustmentModifier for PercentDefaultSloAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultSloAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDefaultSloAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultSloAdjustmentModifier for SourceDefaultSloAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &DefaultSloAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultSloAdjustmentAdjuster {
    pub step: f64,
}

impl DefaultSloAdjustmentModifier for RoundingDefaultSloAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultSloAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default slo adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultSloAdjustmentEngine {
    pub steps: Vec<Rc<dyn DefaultSloAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultSloAdjustmentEngine {
    pub fn evaluate(&mut self, item: &DefaultSloAdjustment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultSloAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_slo_adjustment_engine(flags: &flags::Registry) -> DefaultSloAdjustmentEngine {
    DefaultSloAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedDefaultSloAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "default-slo-adjustment-strict".to_string(),
                percent: -9.8,
            }) as Rc<dyn DefaultSloAdjustmentModifier>,
            Rc::new(PercentDefaultSloAdjustmentAdjuster { percent: -15.2 }),
            Rc::new(SourceDefaultSloAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.97),
                    ("retail".to_string(), 1.32),
                    ("central".to_string(), 0.91),
                ]),
            }),
            Rc::new(RoundingDefaultSloAdjustmentAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 96120.9,
    }
}
