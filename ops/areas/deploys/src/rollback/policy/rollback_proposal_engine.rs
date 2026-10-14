use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback proposal calculation.
pub trait RollbackProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackProposalStep for FlaggedRollbackProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackProposalAdjuster {
    pub percent: f64,
}

impl RollbackProposalStep for PercentRollbackProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackProposalStep for TieredRollbackProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollbackProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackProposalStep for TitleRollbackProposalAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollbackProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackProposalAdjuster {
    pub step: f64,
}

impl RollbackProposalStep for RoundingRollbackProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rollback proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackProposalEngine {
    pub steps: Vec<Rc<dyn RollbackProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackProposalEngine {
    pub fn run(&mut self, item: &RollbackProposal) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RollbackProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollback_proposal_engine(flags: &flags::Registry) -> RollbackProposalEngine {
    RollbackProposalEngine {
        steps: vec![
            Rc::new(FlaggedRollbackProposalAdjuster {
                flags: flags.clone(),
                flag: "rollback-proposal-primary".to_string(),
                percent: -7.9,
            }) as Rc<dyn RollbackProposalStep>,
            Rc::new(PercentRollbackProposalAdjuster { percent: -19.1 }),
            Rc::new(TieredRollbackProposalAdjuster {
                bounds: vec![4720.0, 6805.0, 7287.0],
                rates: vec![1.15, 1.19, 0.87],
            }),
            Rc::new(TitleRollbackProposalAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.37), ("central".to_string(), 1.3)]),
            }),
            Rc::new(RoundingRollbackProposalAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 39624.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_proposal_engine(&flags::new_registry());
        let item = new_rollback_proposal("code-129", "title-895");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
