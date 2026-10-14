use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual verifier assignment calculation.
pub trait ManualVerifierAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualVerifierAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualVerifierAssignmentAdjuster {
    pub percent: f64,
}

impl ManualVerifierAssignmentStep for PercentManualVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualVerifierAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualVerifierAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualVerifierAssignmentStep for TieredManualVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualVerifierAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeManualVerifierAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualVerifierAssignmentStep for ScopeManualVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ManualVerifierAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualVerifierAssignmentAdjuster {
    pub step: f64,
}

impl ManualVerifierAssignmentStep for RoundingManualVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualVerifierAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualVerifierAssignmentAdjuster {
    pub minimum: f64,
}

impl ManualVerifierAssignmentStep for MinimumManualVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualVerifierAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualVerifierAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualVerifierAssignmentStep for FlaggedManualVerifierAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualVerifierAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual verifier assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualVerifierAssignmentEngine {
    pub steps: Vec<Rc<dyn ManualVerifierAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualVerifierAssignmentEngine {
    pub fn price(&mut self, item: &ManualVerifierAssignment) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualVerifierAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_verifier_assignment_engine(flags: &flags::Registry) -> ManualVerifierAssignmentEngine {
    ManualVerifierAssignmentEngine {
        steps: vec![
            Rc::new(PercentManualVerifierAssignmentAdjuster { percent: -6.5 }) as Rc<dyn ManualVerifierAssignmentStep>,
            Rc::new(TieredManualVerifierAssignmentAdjuster {
                bounds: vec![6124.0, 6523.0, 9655.0],
                rates: vec![1.14, 1.04, 1.0],
            }),
            Rc::new(ScopeManualVerifierAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.18),
                    ("south".to_string(), 0.75),
                    ("east".to_string(), 1.07),
                ]),
            }),
            Rc::new(RoundingManualVerifierAssignmentAdjuster { step: 1.0 }),
            Rc::new(MinimumManualVerifierAssignmentAdjuster { minimum: 0.1 }),
            Rc::new(FlaggedManualVerifierAssignmentAdjuster {
                flags: flags.clone(),
                flag: "manual-verifier-assignment-strict".to_string(),
                percent: -9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 34217.9,
    }
}
