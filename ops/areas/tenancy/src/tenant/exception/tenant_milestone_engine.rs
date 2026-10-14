use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant milestone calculation.
pub trait TenantMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantMilestoneAdjuster {
    pub minimum: f64,
}

impl TenantMilestoneFactor for MinimumTenantMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantMilestoneAdjuster {
    pub percent: f64,
}

impl TenantMilestoneFactor for PercentTenantMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantMilestoneAdjuster {
    pub step: f64,
}

impl TenantMilestoneFactor for RoundingTenantMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantMilestoneFactor for TieredTenantMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tenant milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantMilestoneEngine {
    pub steps: Vec<Rc<dyn TenantMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantMilestoneEngine {
    pub fn evaluate(&mut self, item: &TenantMilestone) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantMilestoneFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tenant_milestone_engine() -> TenantMilestoneEngine {
    TenantMilestoneEngine {
        steps: vec![
            Rc::new(MinimumTenantMilestoneAdjuster { minimum: 38.6 }) as Rc<dyn TenantMilestoneFactor>,
            Rc::new(PercentTenantMilestoneAdjuster { percent: -0.0 }),
            Rc::new(RoundingTenantMilestoneAdjuster { step: 0.25 }),
            Rc::new(TieredTenantMilestoneAdjuster {
                bounds: vec![552.0, 877.0, 4440.0, 5713.0],
                rates: vec![0.94, 0.94, 1.17, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 77722.3,
    }
}
