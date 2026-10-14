use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary postmortem adjustment calculation.
pub trait PrimaryPostmortemAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryPostmortemAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryPostmortemAdjustmentAdjuster {
    pub cap: f64,
}

impl PrimaryPostmortemAdjustmentModifier for CapPrimaryPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryPostmortemAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryPostmortemAdjustmentAdjuster {
    pub step: f64,
}

impl PrimaryPostmortemAdjustmentModifier for RoundingPrimaryPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryPostmortemAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemAdjustmentCapacityOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryPostmortemAdjustmentModifier for PrimaryPostmortemAdjustmentCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryPostmortemAdjustment, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryPostmortemAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryPostmortemAdjustmentModifier for TitlePrimaryPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryPostmortemAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary postmortem adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryPostmortemAdjustmentEngine {
    pub steps: Vec<Rc<dyn PrimaryPostmortemAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryPostmortemAdjustmentEngine {
    pub fn evaluate(&mut self, item: &PrimaryPostmortemAdjustment) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryPostmortemAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryPostmortemAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_postmortem_adjustment_engine() -> PrimaryPostmortemAdjustmentEngine {
    PrimaryPostmortemAdjustmentEngine {
        steps: vec![
            Rc::new(CapPrimaryPostmortemAdjustmentAdjuster { cap: 5311.0 })
                as Rc<dyn PrimaryPostmortemAdjustmentModifier>,
            Rc::new(RoundingPrimaryPostmortemAdjustmentAdjuster { step: 0.25 }),
            Rc::new(PrimaryPostmortemAdjustmentCapacityOffsetAdjuster { weight: 0.84 }),
            Rc::new(TitlePrimaryPostmortemAdjustmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.24)]),
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 83313.7,
    }
}
