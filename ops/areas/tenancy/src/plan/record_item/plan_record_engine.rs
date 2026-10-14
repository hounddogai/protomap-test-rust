use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan record calculation.
pub trait PlanRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanRecordAdjuster {
    pub percent: f64,
}

impl PlanRecordStep for PercentPlanRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanRecordStep for TieredPlanRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanRecordAdjuster {
    pub minimum: f64,
}

impl PlanRecordStep for MinimumPlanRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanRecordAdjuster {
    pub step: f64,
}

impl PlanRecordStep for RoundingPlanRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the plan record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanRecordEngine {
    pub steps: Vec<Rc<dyn PlanRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanRecordEngine {
    pub fn compute(&mut self, item: &PlanRecord) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[PlanRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_record_engine() -> PlanRecordEngine {
    PlanRecordEngine {
        steps: vec![
            Rc::new(PercentPlanRecordAdjuster { percent: 21.5 }) as Rc<dyn PlanRecordStep>,
            Rc::new(TieredPlanRecordAdjuster {
                bounds: vec![4262.0, 6341.0],
                rates: vec![1.19, 0.92],
            }),
            Rc::new(MinimumPlanRecordAdjuster { minimum: 37.6 }),
            Rc::new(RoundingPlanRecordAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 93787.0,
    }
}
