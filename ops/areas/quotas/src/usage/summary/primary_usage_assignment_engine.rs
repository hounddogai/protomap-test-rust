use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the primary usage assignment calculation.
pub trait PrimaryUsageAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryUsageAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryUsageAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryUsageAssignmentFactor for TieredPrimaryUsageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryUsageAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePrimaryUsageAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryUsageAssignmentFactor for NamespacePrimaryUsageAssignmentAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &PrimaryUsageAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryUsageAssignmentAdjuster {
    pub percent: f64,
}

impl PrimaryUsageAssignmentFactor for PercentPrimaryUsageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryUsageAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary usage assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryUsageAssignmentEngine {
    pub steps: Vec<Rc<dyn PrimaryUsageAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryUsageAssignmentEngine {
    pub fn run(&mut self, item: &PrimaryUsageAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryUsageAssignmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryUsageAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_usage_assignment_engine() -> PrimaryUsageAssignmentEngine {
    PrimaryUsageAssignmentEngine {
        steps: vec![
            Rc::new(TieredPrimaryUsageAssignmentAdjuster {
                bounds: vec![5579.0, 8166.0],
                rates: vec![1.14, 0.88],
            }) as Rc<dyn PrimaryUsageAssignmentFactor>,
            Rc::new(NamespacePrimaryUsageAssignmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.23)]),
            }),
            Rc::new(PercentPrimaryUsageAssignmentAdjuster { percent: 22.7 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 6262.6,
    }
}
