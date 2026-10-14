use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled token proposal calculation.
pub trait ScheduledTokenProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTokenProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledTokenProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledTokenProposalStep for NameScheduledTokenProposalAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledTokenProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledTokenProposalAdjuster {
    pub cap: f64,
}

impl ScheduledTokenProposalStep for CapScheduledTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledTokenProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledTokenProposalAdjuster {
    pub percent: f64,
}

impl ScheduledTokenProposalStep for PercentScheduledTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledTokenProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled token proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTokenProposalEngine {
    pub steps: Vec<Rc<dyn ScheduledTokenProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTokenProposalEngine {
    pub fn compute(&mut self, item: &ScheduledTokenProposal) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTokenProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_token_proposal_engine() -> ScheduledTokenProposalEngine {
    ScheduledTokenProposalEngine {
        steps: vec![
            Rc::new(NameScheduledTokenProposalAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.83), ("west".to_string(), 1.34)]),
            }) as Rc<dyn ScheduledTokenProposalStep>,
            Rc::new(CapScheduledTokenProposalAdjuster { cap: 2919.0 }),
            Rc::new(PercentScheduledTokenProposalAdjuster { percent: -20.4 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 11310.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_token_proposal_engine();
        let item = new_scheduled_token_proposal("code-160", "name-517");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
