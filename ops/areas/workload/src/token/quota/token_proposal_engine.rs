use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the token proposal calculation.
pub trait TokenProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTokenProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenProposalStep for CategoryTokenProposalAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &TokenProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenProposalStep for TieredTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTokenProposalAdjuster {
    pub step: f64,
}

impl TokenProposalStep for RoundingTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TokenProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTokenProposalAdjuster {
    pub minimum: f64,
}

impl TokenProposalStep for MinimumTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TokenProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the token proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenProposalEngine {
    pub steps: Vec<Rc<dyn TokenProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenProposalEngine {
    pub fn run(&mut self, item: &TokenProposal) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[TokenProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_token_proposal_engine() -> TokenProposalEngine {
    TokenProposalEngine {
        steps: vec![
            Rc::new(CategoryTokenProposalAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.75)]),
            }) as Rc<dyn TokenProposalStep>,
            Rc::new(TieredTokenProposalAdjuster {
                bounds: vec![3360.0, 7410.0],
                rates: vec![1.16, 1.07],
            }),
            Rc::new(RoundingTokenProposalAdjuster { step: 0.5 }),
            Rc::new(MinimumTokenProposalAdjuster { minimum: 27.9 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 87784.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_proposal_engine();
        let item = new_token_proposal("code-660", "name-797");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
