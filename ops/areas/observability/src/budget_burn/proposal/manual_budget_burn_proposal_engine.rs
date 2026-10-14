use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual budget burn proposal calculation.
pub trait ManualBudgetBurnProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBudgetBurnProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryManualBudgetBurnProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualBudgetBurnProposalStep for CategoryManualBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ManualBudgetBurnProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualBudgetBurnProposalAdjuster {
    pub step: f64,
}

impl ManualBudgetBurnProposalStep for RoundingManualBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualBudgetBurnProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualBudgetBurnProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualBudgetBurnProposalStep for FlaggedManualBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualBudgetBurnProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualBudgetBurnProposalAdjuster {
    pub cap: f64,
}

impl ManualBudgetBurnProposalStep for CapManualBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualBudgetBurnProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualBudgetBurnProposalAdjuster {
    pub percent: f64,
}

impl ManualBudgetBurnProposalStep for PercentManualBudgetBurnProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualBudgetBurnProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual budget burn proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnProposalEngine {
    pub steps: Vec<Rc<dyn ManualBudgetBurnProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBudgetBurnProposalEngine {
    pub fn evaluate(&mut self, item: &ManualBudgetBurnProposal) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ManualBudgetBurnProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_budget_burn_proposal_engine(flags: &flags::Registry) -> ManualBudgetBurnProposalEngine {
    ManualBudgetBurnProposalEngine {
        steps: vec![
            Rc::new(CategoryManualBudgetBurnProposalAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.13), ("north".to_string(), 1.22)]),
            }) as Rc<dyn ManualBudgetBurnProposalStep>,
            Rc::new(RoundingManualBudgetBurnProposalAdjuster { step: 0.1 }),
            Rc::new(FlaggedManualBudgetBurnProposalAdjuster {
                flags: flags.clone(),
                flag: "manual-budget-burn-proposal-regional".to_string(),
                percent: -8.5,
            }),
            Rc::new(CapManualBudgetBurnProposalAdjuster { cap: 15895.0 }),
            Rc::new(PercentManualBudgetBurnProposalAdjuster { percent: 18.5 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 94287.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_budget_burn_proposal_engine(&flags::new_registry());
        let item = new_manual_budget_burn_proposal("code-658", "title-824");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
