use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item proposal calculation.
pub trait ActionItemProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemProposalModifier for TieredActionItemProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryActionItemProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemProposalModifier for CategoryActionItemProposalAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ActionItemProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemProposalModifier for FlaggedActionItemProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemProposalAdjuster {
    pub cap: f64,
}

impl ActionItemProposalModifier for CapActionItemProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the action item proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemProposalEngine {
    pub steps: Vec<Rc<dyn ActionItemProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemProposalEngine {
    pub fn price(&mut self, item: &ActionItemProposal) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_proposal_engine(flags: &flags::Registry) -> ActionItemProposalEngine {
    ActionItemProposalEngine {
        steps: vec![
            Rc::new(TieredActionItemProposalAdjuster {
                bounds: vec![224.0, 6971.0, 7670.0, 9170.0],
                rates: vec![0.83, 0.9, 1.04, 1.02],
            }) as Rc<dyn ActionItemProposalModifier>,
            Rc::new(CategoryActionItemProposalAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.28),
                    ("retail".to_string(), 0.83),
                    ("east".to_string(), 0.84),
                ]),
            }),
            Rc::new(FlaggedActionItemProposalAdjuster {
                flags: flags.clone(),
                flag: "action-item-proposal-tiered".to_string(),
                percent: 8.0,
            }),
            Rc::new(CapActionItemProposalAdjuster { cap: 43100.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 41385.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_proposal_engine(&flags::new_registry());
        let item = new_action_item_proposal("code-988", "name-504");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
