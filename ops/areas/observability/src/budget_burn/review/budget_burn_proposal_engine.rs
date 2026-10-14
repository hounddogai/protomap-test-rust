use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn proposal calculation.
pub trait BudgetBurnProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnProposalAdjuster {
    pub step: f64,
}

impl BudgetBurnProposalAdjuster for RoundingBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnProposalAdjuster for FlaggedBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnProposalAdjuster for TieredBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceBudgetBurnProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnProposalAdjuster for SourceBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &BudgetBurnProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnProposalAdjuster {
    pub cap: f64,
}

impl BudgetBurnProposalAdjuster for CapBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the budget burn proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnProposalEngine {
    pub steps: Vec<Rc<dyn BudgetBurnProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnProposalEngine {
    pub fn run(&mut self, item: &BudgetBurnProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnProposalAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_budget_burn_proposal_engine(flags: &flags::Registry) -> BudgetBurnProposalEngine {
    BudgetBurnProposalEngine {
        steps: vec![
            Rc::new(RoundingBudgetBurnProposalAdjuster { step: 1.0 }) as Rc<dyn BudgetBurnProposalAdjuster>,
            Rc::new(FlaggedBudgetBurnProposalAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-proposal-regional".to_string(),
                percent: 7.4,
            }),
            Rc::new(TieredBudgetBurnProposalAdjuster {
                bounds: vec![2456.0, 4936.0],
                rates: vec![0.98, 1.16],
            }),
            Rc::new(SourceBudgetBurnProposalAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.04),
                    ("central".to_string(), 1.15),
                    ("west".to_string(), 0.97),
                ]),
            }),
            Rc::new(CapBudgetBurnProposalAdjuster { cap: 31167.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 14028.2,
    }
}
