use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the federation milestone calculation.
pub trait FederationMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationMilestoneAdjuster {
    pub cap: f64,
}

impl FederationMilestoneStep for CapFederationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationMilestoneStep for TieredFederationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationMilestoneAdjuster {
    pub minimum: f64,
}

impl FederationMilestoneStep for MinimumFederationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationMilestoneStep for FlaggedFederationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the federation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationMilestoneEngine {
    pub steps: Vec<Rc<dyn FederationMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationMilestoneEngine {
    pub fn price(&mut self, item: &FederationMilestone) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_milestone_engine(flags: &flags::Registry) -> FederationMilestoneEngine {
    FederationMilestoneEngine {
        steps: vec![
            Rc::new(CapFederationMilestoneAdjuster { cap: 28321.0 }) as Rc<dyn FederationMilestoneStep>,
            Rc::new(TieredFederationMilestoneAdjuster {
                bounds: vec![130.0, 4516.0, 9913.0],
                rates: vec![0.98, 0.82, 0.91],
            }),
            Rc::new(MinimumFederationMilestoneAdjuster { minimum: 28.5 }),
            Rc::new(FlaggedFederationMilestoneAdjuster {
                flags: flags.clone(),
                flag: "federation-milestone-strict".to_string(),
                percent: 2.0,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 32325.0,
    }
}
