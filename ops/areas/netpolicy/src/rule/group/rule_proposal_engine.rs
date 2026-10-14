use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule proposal calculation.
pub trait RuleProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRuleProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleProposalFactor for SegmentRuleProposalAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RuleProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleProposalAdjuster {
    pub step: f64,
}

impl RuleProposalFactor for RoundingRuleProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleProposalFactor for TieredRuleProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleProposalAdjuster {
    pub cap: f64,
}

impl RuleProposalFactor for CapRuleProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rule proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleProposalEngine {
    pub steps: Vec<Rc<dyn RuleProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleProposalEngine {
    pub fn run(&mut self, item: &RuleProposal) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RuleProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rule_proposal_engine() -> RuleProposalEngine {
    RuleProposalEngine {
        steps: vec![
            Rc::new(SegmentRuleProposalAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.71)]),
            }) as Rc<dyn RuleProposalFactor>,
            Rc::new(RoundingRuleProposalAdjuster { step: 0.1 }),
            Rc::new(TieredRuleProposalAdjuster {
                bounds: vec![406.0, 4341.0],
                rates: vec![0.99, 0.93],
            }),
            Rc::new(CapRuleProposalAdjuster { cap: 49015.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 23406.8,
    }
}
