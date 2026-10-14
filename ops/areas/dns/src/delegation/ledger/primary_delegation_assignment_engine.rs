use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary delegation assignment calculation.
pub trait PrimaryDelegationAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryDelegationAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryDelegationAssignmentAdjuster {
    pub step: f64,
}

impl PrimaryDelegationAssignmentAdjuster for RoundingPrimaryDelegationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryDelegationAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryDelegationAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryDelegationAssignmentAdjuster for FlaggedPrimaryDelegationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryDelegationAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryDelegationAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryDelegationAssignmentAdjuster for TieredPrimaryDelegationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryDelegationAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimaryDelegationAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryDelegationAssignmentAdjuster for ClusterPrimaryDelegationAssignmentAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimaryDelegationAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary delegation assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryDelegationAssignmentEngine {
    pub steps: Vec<Rc<dyn PrimaryDelegationAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryDelegationAssignmentEngine {
    pub fn run(&mut self, item: &PrimaryDelegationAssignment) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryDelegationAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_delegation_assignment_engine(flags: &flags::Registry) -> PrimaryDelegationAssignmentEngine {
    PrimaryDelegationAssignmentEngine {
        steps: vec![
            Rc::new(RoundingPrimaryDelegationAssignmentAdjuster { step: 0.05 })
                as Rc<dyn PrimaryDelegationAssignmentAdjuster>,
            Rc::new(FlaggedPrimaryDelegationAssignmentAdjuster {
                flags: flags.clone(),
                flag: "primary-delegation-assignment-fallback".to_string(),
                percent: 6.8,
            }),
            Rc::new(TieredPrimaryDelegationAssignmentAdjuster {
                bounds: vec![4131.0, 4351.0, 4354.0, 5621.0],
                rates: vec![1.17, 0.9, 1.17, 1.0],
            }),
            Rc::new(ClusterPrimaryDelegationAssignmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.94), ("east".to_string(), 1.24)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 58482.0,
    }
}
