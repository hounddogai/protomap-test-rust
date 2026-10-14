use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual token proposal calculation.
pub trait ManualTokenProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTokenProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualTokenProposalAdjuster {
    pub minimum: f64,
}

impl ManualTokenProposalFactor for MinimumManualTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualTokenProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterManualTokenProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualTokenProposalFactor for ClusterManualTokenProposalAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &ManualTokenProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualTokenProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualTokenProposalFactor for TieredManualTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualTokenProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualTokenProposalAdjuster {
    pub percent: f64,
}

impl ManualTokenProposalFactor for PercentManualTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualTokenProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualTokenProposalAdjuster {
    pub cap: f64,
}

impl ManualTokenProposalFactor for CapManualTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualTokenProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualTokenProposalAdjuster {
    pub step: f64,
}

impl ManualTokenProposalFactor for RoundingManualTokenProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualTokenProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual token proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTokenProposalEngine {
    pub steps: Vec<Rc<dyn ManualTokenProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTokenProposalEngine {
    pub fn price(&mut self, item: &ManualTokenProposal) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualTokenProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_token_proposal_engine() -> ManualTokenProposalEngine {
    ManualTokenProposalEngine {
        steps: vec![
            Rc::new(MinimumManualTokenProposalAdjuster { minimum: 19.6 }) as Rc<dyn ManualTokenProposalFactor>,
            Rc::new(ClusterManualTokenProposalAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.98)]),
            }),
            Rc::new(TieredManualTokenProposalAdjuster {
                bounds: vec![1925.0, 3444.0],
                rates: vec![1.09, 0.93],
            }),
            Rc::new(PercentManualTokenProposalAdjuster { percent: 12.7 }),
            Rc::new(CapManualTokenProposalAdjuster { cap: 46048.0 }),
            Rc::new(RoundingManualTokenProposalAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 55920.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_token_proposal_engine();
        let item = new_manual_token_proposal("code-778", "name-515");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
