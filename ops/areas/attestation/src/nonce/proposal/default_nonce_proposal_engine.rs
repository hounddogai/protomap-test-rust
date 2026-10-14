use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default nonce proposal calculation.
pub trait DefaultNonceProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultNonceProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceProposalDiscountOffsetAdjuster {
    pub weight: f64,
}

impl DefaultNonceProposalAdjuster for DefaultNonceProposalDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultNonceProposal, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultNonceProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultNonceProposalAdjuster for TieredDefaultNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultNonceProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultNonceProposalAdjuster {
    pub step: f64,
}

impl DefaultNonceProposalAdjuster for RoundingDefaultNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultNonceProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleDefaultNonceProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultNonceProposalAdjuster for LocaleDefaultNonceProposalAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &DefaultNonceProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultNonceProposalAdjuster {
    pub minimum: f64,
}

impl DefaultNonceProposalAdjuster for MinimumDefaultNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultNonceProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultNonceProposalAdjuster {
    pub cap: f64,
}

impl DefaultNonceProposalAdjuster for CapDefaultNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultNonceProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default nonce proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultNonceProposalEngine {
    pub steps: Vec<Rc<dyn DefaultNonceProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultNonceProposalEngine {
    pub fn evaluate(&mut self, item: &DefaultNonceProposal) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultNonceProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_nonce_proposal_engine() -> DefaultNonceProposalEngine {
    DefaultNonceProposalEngine {
        steps: vec![
            Rc::new(DefaultNonceProposalDiscountOffsetAdjuster { weight: 0.49 })
                as Rc<dyn DefaultNonceProposalAdjuster>,
            Rc::new(TieredDefaultNonceProposalAdjuster {
                bounds: vec![2561.0, 7244.0, 7881.0],
                rates: vec![1.04, 1.1, 1.04],
            }),
            Rc::new(RoundingDefaultNonceProposalAdjuster { step: 0.25 }),
            Rc::new(LocaleDefaultNonceProposalAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.76), ("south".to_string(), 1.14)]),
            }),
            Rc::new(MinimumDefaultNonceProposalAdjuster { minimum: 21.0 }),
            Rc::new(CapDefaultNonceProposalAdjuster { cap: 6188.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 28078.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_nonce_proposal_engine();
        let item = new_default_nonce_proposal("code-119", "label-957");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
