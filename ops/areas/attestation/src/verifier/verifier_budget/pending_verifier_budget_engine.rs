use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending verifier budget calculation.
pub trait PendingVerifierBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingVerifierBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingVerifierBudgetAdjuster {
    pub minimum: f64,
}

impl PendingVerifierBudgetAdjuster for MinimumPendingVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingVerifierBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingVerifierBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingVerifierBudgetAdjuster for FlaggedPendingVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingVerifierBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingVerifierBudgetAdjuster {
    pub percent: f64,
}

impl PendingVerifierBudgetAdjuster for PercentPendingVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingVerifierBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePendingVerifierBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingVerifierBudgetAdjuster for LocalePendingVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PendingVerifierBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending verifier budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingVerifierBudgetEngine {
    pub steps: Vec<Rc<dyn PendingVerifierBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingVerifierBudgetEngine {
    pub fn compute(&mut self, item: &PendingVerifierBudget) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingVerifierBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingVerifierBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_verifier_budget_engine(flags: &flags::Registry) -> PendingVerifierBudgetEngine {
    PendingVerifierBudgetEngine {
        steps: vec![
            Rc::new(MinimumPendingVerifierBudgetAdjuster { minimum: 22.6 }) as Rc<dyn PendingVerifierBudgetAdjuster>,
            Rc::new(FlaggedPendingVerifierBudgetAdjuster {
                flags: flags.clone(),
                flag: "pending-verifier-budget-legacy".to_string(),
                percent: -3.3,
            }),
            Rc::new(PercentPendingVerifierBudgetAdjuster { percent: 10.2 }),
            Rc::new(LocalePendingVerifierBudgetAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.04), ("online".to_string(), 0.79)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 7399.6,
    }
}
