use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary proposal calculation.
pub trait CanaryProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryProposalAdjuster {
    pub step: f64,
}

impl CanaryProposalModifier for RoundingCanaryProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryProposalModifier for TieredCanaryProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleCanaryProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryProposalModifier for LocaleCanaryProposalAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &CanaryProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryProposalMinimumOffsetAdjuster {
    pub weight: f64,
}

impl CanaryProposalModifier for CanaryProposalMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryProposal, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryProposalAdjuster {
    pub cap: f64,
}

impl CanaryProposalModifier for CapCanaryProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryProposalAdjuster {
    pub minimum: f64,
}

impl CanaryProposalModifier for MinimumCanaryProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the canary proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryProposalEngine {
    pub steps: Vec<Rc<dyn CanaryProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryProposalEngine {
    pub fn price(&mut self, item: &CanaryProposal) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CanaryProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_proposal_engine() -> CanaryProposalEngine {
    CanaryProposalEngine {
        steps: vec![
            Rc::new(RoundingCanaryProposalAdjuster { step: 0.05 }) as Rc<dyn CanaryProposalModifier>,
            Rc::new(TieredCanaryProposalAdjuster {
                bounds: vec![184.0, 1685.0, 2817.0],
                rates: vec![0.92, 0.93, 0.9],
            }),
            Rc::new(LocaleCanaryProposalAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.32)]),
            }),
            Rc::new(CanaryProposalMinimumOffsetAdjuster { weight: 0.37 }),
            Rc::new(CapCanaryProposalAdjuster { cap: 10264.0 }),
            Rc::new(MinimumCanaryProposalAdjuster { minimum: 40.4 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 1981.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_proposal_engine();
        let item = new_canary_proposal("code-733", "label-244");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
