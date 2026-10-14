use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient suspension assignment calculation.
pub trait LenientSuspensionAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSuspensionAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientSuspensionAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientSuspensionAssignmentFactor for FlaggedLenientSuspensionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientSuspensionAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientSuspensionAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientSuspensionAssignmentFactor for TieredLenientSuspensionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientSuspensionAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSuspensionAssignmentAdjuster {
    pub cap: f64,
}

impl LenientSuspensionAssignmentFactor for CapLenientSuspensionAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSuspensionAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient suspension assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSuspensionAssignmentEngine {
    pub steps: Vec<Rc<dyn LenientSuspensionAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSuspensionAssignmentEngine {
    pub fn run(&mut self, item: &LenientSuspensionAssignment) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[LenientSuspensionAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_suspension_assignment_engine(flags: &flags::Registry) -> LenientSuspensionAssignmentEngine {
    LenientSuspensionAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedLenientSuspensionAssignmentAdjuster {
                flags: flags.clone(),
                flag: "lenient-suspension-assignment-primary".to_string(),
                percent: 0.9,
            }) as Rc<dyn LenientSuspensionAssignmentFactor>,
            Rc::new(TieredLenientSuspensionAssignmentAdjuster {
                bounds: vec![6253.0, 6637.0],
                rates: vec![1.17, 0.9],
            }),
            Rc::new(CapLenientSuspensionAssignmentAdjuster { cap: 25627.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 72902.6,
    }
}
