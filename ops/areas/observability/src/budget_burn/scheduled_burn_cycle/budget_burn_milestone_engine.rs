use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn milestone calculation.
pub trait BudgetBurnMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnMilestoneAdjuster {
    pub cap: f64,
}

impl BudgetBurnMilestoneStep for CapBudgetBurnMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnMilestoneAdjuster {
    pub step: f64,
}

impl BudgetBurnMilestoneStep for RoundingBudgetBurnMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnMilestoneAdjuster {
    pub percent: f64,
}

impl BudgetBurnMilestoneStep for PercentBudgetBurnMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnMilestoneStep for FlaggedBudgetBurnMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnMilestoneStep for TieredBudgetBurnMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBudgetBurnMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnMilestoneStep for NameBudgetBurnMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BudgetBurnMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the budget burn milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnMilestoneEngine {
    pub steps: Vec<Rc<dyn BudgetBurnMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnMilestoneEngine {
    pub fn price(&mut self, item: &BudgetBurnMilestone) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_milestone_engine(flags: &flags::Registry) -> BudgetBurnMilestoneEngine {
    BudgetBurnMilestoneEngine {
        steps: vec![
            Rc::new(CapBudgetBurnMilestoneAdjuster { cap: 46754.0 }) as Rc<dyn BudgetBurnMilestoneStep>,
            Rc::new(RoundingBudgetBurnMilestoneAdjuster { step: 5.0 }),
            Rc::new(PercentBudgetBurnMilestoneAdjuster { percent: 22.9 }),
            Rc::new(FlaggedBudgetBurnMilestoneAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-milestone-batched".to_string(),
                percent: 3.6,
            }),
            Rc::new(TieredBudgetBurnMilestoneAdjuster {
                bounds: vec![1327.0, 2563.0, 7270.0, 9832.0],
                rates: vec![1.01, 1.05, 0.96, 0.87],
            }),
            Rc::new(NameBudgetBurnMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.22)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 56781.2,
    }
}
