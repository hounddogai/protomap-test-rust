use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy nonce proposal calculation.
pub trait LegacyNonceProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyNonceProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyNonceProposalAdjuster {
    pub percent: f64,
}

impl LegacyNonceProposalAdjuster for PercentLegacyNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyNonceProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyNonceProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyNonceProposalAdjuster for TieredLegacyNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyNonceProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceProposalUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyNonceProposalAdjuster for LegacyNonceProposalUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyNonceProposal, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyNonceProposalAdjuster {
    pub minimum: f64,
}

impl LegacyNonceProposalAdjuster for MinimumLegacyNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyNonceProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyNonceProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyNonceProposalAdjuster for FlaggedLegacyNonceProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyNonceProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy nonce proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyNonceProposalEngine {
    pub steps: Vec<Rc<dyn LegacyNonceProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyNonceProposalEngine {
    pub fn evaluate(&mut self, item: &LegacyNonceProposal) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyNonceProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_nonce_proposal_engine(flags: &flags::Registry) -> LegacyNonceProposalEngine {
    LegacyNonceProposalEngine {
        steps: vec![
            Rc::new(PercentLegacyNonceProposalAdjuster { percent: -14.6 }) as Rc<dyn LegacyNonceProposalAdjuster>,
            Rc::new(TieredLegacyNonceProposalAdjuster {
                bounds: vec![240.0, 6164.0],
                rates: vec![0.81, 1.1],
            }),
            Rc::new(LegacyNonceProposalUpdatedAtOffsetAdjuster { weight: 0.33 }),
            Rc::new(MinimumLegacyNonceProposalAdjuster { minimum: 0.9 }),
            Rc::new(FlaggedLegacyNonceProposalAdjuster {
                flags: flags.clone(),
                flag: "legacy-nonce-proposal-pending".to_string(),
                percent: -1.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 67553.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_nonce_proposal_engine(&flags::new_registry());
        let item = new_legacy_nonce_proposal("code-362", "label-137");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
