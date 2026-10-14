use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled trust domain budget calculation.
pub trait ScheduledTrustDomainBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTrustDomainBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledTrustDomainBudgetAdjuster {
    pub step: f64,
}

impl ScheduledTrustDomainBudgetStep for RoundingScheduledTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledTrustDomainBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledTrustDomainBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledTrustDomainBudgetStep for TieredScheduledTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledTrustDomainBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledTrustDomainBudgetAdjuster {
    pub cap: f64,
}

impl ScheduledTrustDomainBudgetStep for CapScheduledTrustDomainBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledTrustDomainBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainBudgetStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledTrustDomainBudgetStep for ScheduledTrustDomainBudgetStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledTrustDomainBudget, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the scheduled trust domain budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTrustDomainBudgetEngine {
    pub steps: Vec<Rc<dyn ScheduledTrustDomainBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTrustDomainBudgetEngine {
    pub fn compute(&mut self, item: &ScheduledTrustDomainBudget) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledTrustDomainBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTrustDomainBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_trust_domain_budget_engine() -> ScheduledTrustDomainBudgetEngine {
    ScheduledTrustDomainBudgetEngine {
        steps: vec![
            Rc::new(RoundingScheduledTrustDomainBudgetAdjuster { step: 1.0 }) as Rc<dyn ScheduledTrustDomainBudgetStep>,
            Rc::new(TieredScheduledTrustDomainBudgetAdjuster {
                bounds: vec![1988.0, 7395.0, 8128.0],
                rates: vec![0.86, 1.01, 1.17],
            }),
            Rc::new(CapScheduledTrustDomainBudgetAdjuster { cap: 15241.0 }),
            Rc::new(ScheduledTrustDomainBudgetStartsAtOffsetAdjuster { weight: 0.35 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 99583.5,
    }
}
