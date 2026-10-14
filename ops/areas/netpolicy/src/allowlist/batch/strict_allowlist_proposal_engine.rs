use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict allowlist proposal calculation.
pub trait StrictAllowlistProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAllowlistProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictAllowlistProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictAllowlistProposalAdjuster for TieredStrictAllowlistProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictAllowlistProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistProposalMaximumOffsetAdjuster {
    pub weight: f64,
}

impl StrictAllowlistProposalAdjuster for StrictAllowlistProposalMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &StrictAllowlistProposal, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelStrictAllowlistProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictAllowlistProposalAdjuster for LabelStrictAllowlistProposalAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &StrictAllowlistProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict allowlist proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAllowlistProposalEngine {
    pub steps: Vec<Rc<dyn StrictAllowlistProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAllowlistProposalEngine {
    pub fn price(&mut self, item: &StrictAllowlistProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictAllowlistProposalAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictAllowlistProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_allowlist_proposal_engine() -> StrictAllowlistProposalEngine {
    StrictAllowlistProposalEngine {
        steps: vec![
            Rc::new(TieredStrictAllowlistProposalAdjuster {
                bounds: vec![2175.0, 9236.0, 9413.0],
                rates: vec![0.96, 0.88, 1.1],
            }) as Rc<dyn StrictAllowlistProposalAdjuster>,
            Rc::new(StrictAllowlistProposalMaximumOffsetAdjuster { weight: 0.93 }),
            Rc::new(LabelStrictAllowlistProposalAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.1), ("retail".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 62696.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_allowlist_proposal_engine();
        let item = new_strict_allowlist_proposal("code-837", "label-74");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
