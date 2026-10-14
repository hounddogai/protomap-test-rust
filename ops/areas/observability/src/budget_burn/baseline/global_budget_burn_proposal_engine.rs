use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global budget burn proposal calculation.
pub trait GlobalBudgetBurnProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalBudgetBurnProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalBudgetBurnProposalAdjuster {
    pub percent: f64,
}

impl GlobalBudgetBurnProposalFactor for PercentGlobalBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalBudgetBurnProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalBudgetBurnProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalBudgetBurnProposalFactor for TieredGlobalBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalBudgetBurnProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalBudgetBurnProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalBudgetBurnProposalFactor for FlaggedGlobalBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalBudgetBurnProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalBudgetBurnProposalAdjuster {
    pub step: f64,
}

impl GlobalBudgetBurnProposalFactor for RoundingGlobalBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalBudgetBurnProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global budget burn proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalBudgetBurnProposalEngine {
    pub steps: Vec<Rc<dyn GlobalBudgetBurnProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalBudgetBurnProposalEngine {
    pub fn evaluate(&mut self, item: &GlobalBudgetBurnProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalBudgetBurnProposalFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalBudgetBurnProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_budget_burn_proposal_engine(flags: &flags::Registry) -> GlobalBudgetBurnProposalEngine {
    GlobalBudgetBurnProposalEngine {
        steps: vec![
            Rc::new(PercentGlobalBudgetBurnProposalAdjuster { percent: 8.5 }) as Rc<dyn GlobalBudgetBurnProposalFactor>,
            Rc::new(TieredGlobalBudgetBurnProposalAdjuster {
                bounds: vec![8625.0, 8662.0],
                rates: vec![1.13, 1.1],
            }),
            Rc::new(FlaggedGlobalBudgetBurnProposalAdjuster {
                flags: flags.clone(),
                flag: "global-budget-burn-proposal-capped".to_string(),
                percent: 4.3,
            }),
            Rc::new(RoundingGlobalBudgetBurnProposalAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 22621.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_budget_burn_proposal_engine(&flags::new_registry());
        let item = new_global_budget_burn_proposal("code-71", "name-804");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
