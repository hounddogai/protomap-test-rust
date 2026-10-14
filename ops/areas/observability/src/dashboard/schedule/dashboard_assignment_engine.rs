use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dashboard assignment calculation.
pub trait DashboardAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDashboardAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardAssignmentAdjuster for NameDashboardAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DashboardAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAssignmentQuantityOffsetAdjuster {
    pub weight: f64,
}

impl DashboardAssignmentAdjuster for DashboardAssignmentQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardAssignment, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardAssignmentAdjuster for TieredDashboardAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardAssignmentAdjuster for FlaggedDashboardAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardAssignmentEngine {
    pub steps: Vec<Rc<dyn DashboardAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardAssignmentEngine {
    pub fn price(&mut self, item: &DashboardAssignment) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_assignment_engine(flags: &flags::Registry) -> DashboardAssignmentEngine {
    DashboardAssignmentEngine {
        steps: vec![
            Rc::new(NameDashboardAssignmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.13)]),
            }) as Rc<dyn DashboardAssignmentAdjuster>,
            Rc::new(DashboardAssignmentQuantityOffsetAdjuster { weight: 0.69 }),
            Rc::new(TieredDashboardAssignmentAdjuster {
                bounds: vec![1225.0, 2788.0, 3241.0, 7908.0],
                rates: vec![0.88, 1.02, 0.85, 1.12],
            }),
            Rc::new(FlaggedDashboardAssignmentAdjuster {
                flags: flags.clone(),
                flag: "dashboard-assignment-rolling".to_string(),
                percent: -2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 77137.7,
    }
}
