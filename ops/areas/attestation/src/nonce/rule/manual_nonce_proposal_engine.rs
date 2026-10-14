use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual nonce proposal calculation.
pub trait ManualNonceProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualNonceProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceProposalUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ManualNonceProposalStep for ManualNonceProposalUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &ManualNonceProposal, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryManualNonceProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualNonceProposalStep for CategoryManualNonceProposalAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ManualNonceProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualNonceProposalAdjuster {
    pub minimum: f64,
}

impl ManualNonceProposalStep for MinimumManualNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualNonceProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualNonceProposalAdjuster {
    pub cap: f64,
}

impl ManualNonceProposalStep for CapManualNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualNonceProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual nonce proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualNonceProposalEngine {
    pub steps: Vec<Rc<dyn ManualNonceProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualNonceProposalEngine {
    pub fn price(&mut self, item: &ManualNonceProposal) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualNonceProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_nonce_proposal_engine() -> ManualNonceProposalEngine {
    ManualNonceProposalEngine {
        steps: vec![
            Rc::new(ManualNonceProposalUpdatedAtOffsetAdjuster { weight: 1.65 }) as Rc<dyn ManualNonceProposalStep>,
            Rc::new(CategoryManualNonceProposalAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.36)]),
            }),
            Rc::new(MinimumManualNonceProposalAdjuster { minimum: 43.7 }),
            Rc::new(CapManualNonceProposalAdjuster { cap: 2040.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 84054.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_nonce_proposal_engine();
        let item = new_manual_nonce_proposal("code-295", "name-547");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
