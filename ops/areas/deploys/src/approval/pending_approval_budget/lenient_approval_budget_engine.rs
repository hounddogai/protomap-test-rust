use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient approval budget calculation.
pub trait LenientApprovalBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientApprovalBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientApprovalBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientApprovalBudgetAdjuster for TieredLenientApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientApprovalBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientApprovalBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientApprovalBudgetAdjuster for FlaggedLenientApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientApprovalBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientApprovalBudgetAdjuster {
    pub minimum: f64,
}

impl LenientApprovalBudgetAdjuster for MinimumLenientApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientApprovalBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientApprovalBudgetAdjuster {
    pub percent: f64,
}

impl LenientApprovalBudgetAdjuster for PercentLenientApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientApprovalBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalBudgetRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl LenientApprovalBudgetAdjuster for LenientApprovalBudgetRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &LenientApprovalBudget, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientApprovalBudgetAdjuster {
    pub cap: f64,
}

impl LenientApprovalBudgetAdjuster for CapLenientApprovalBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientApprovalBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient approval budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientApprovalBudgetEngine {
    pub steps: Vec<Rc<dyn LenientApprovalBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientApprovalBudgetEngine {
    pub fn price(&mut self, item: &LenientApprovalBudget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientApprovalBudgetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientApprovalBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_approval_budget_engine(flags: &flags::Registry) -> LenientApprovalBudgetEngine {
    LenientApprovalBudgetEngine {
        steps: vec![
            Rc::new(TieredLenientApprovalBudgetAdjuster {
                bounds: vec![3011.0, 4988.0, 6239.0, 7916.0],
                rates: vec![0.93, 0.92, 0.87, 0.94],
            }) as Rc<dyn LenientApprovalBudgetAdjuster>,
            Rc::new(FlaggedLenientApprovalBudgetAdjuster {
                flags: flags.clone(),
                flag: "lenient-approval-budget-regional".to_string(),
                percent: 7.2,
            }),
            Rc::new(MinimumLenientApprovalBudgetAdjuster { minimum: 6.4 }),
            Rc::new(PercentLenientApprovalBudgetAdjuster { percent: -22.6 }),
            Rc::new(LenientApprovalBudgetRotationDaysOffsetAdjuster { weight: 0.82 }),
            Rc::new(CapLenientApprovalBudgetAdjuster { cap: 7448.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 61863.9,
    }
}
