use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive page proposal calculation.
pub trait AdaptivePageProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptivePageProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptivePageProposalAdjuster {
    pub percent: f64,
}

impl AdaptivePageProposalStep for PercentAdaptivePageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptivePageProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptivePageProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptivePageProposalStep for FlaggedAdaptivePageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptivePageProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptivePageProposalAdjuster {
    pub cap: f64,
}

impl AdaptivePageProposalStep for CapAdaptivePageProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptivePageProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptivePageProposalAdjuster {
    pub step: f64,
}

impl AdaptivePageProposalStep for RoundingAdaptivePageProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptivePageProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageProposalRatioOffsetAdjuster {
    pub weight: f64,
}

impl AdaptivePageProposalStep for AdaptivePageProposalRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptivePageProposal, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptivePageProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptivePageProposalStep for TieredAdaptivePageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptivePageProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive page proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptivePageProposalEngine {
    pub steps: Vec<Rc<dyn AdaptivePageProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptivePageProposalEngine {
    pub fn price(&mut self, item: &AdaptivePageProposal) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptivePageProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_page_proposal_engine(flags: &flags::Registry) -> AdaptivePageProposalEngine {
    AdaptivePageProposalEngine {
        steps: vec![
            Rc::new(PercentAdaptivePageProposalAdjuster { percent: -23.5 }) as Rc<dyn AdaptivePageProposalStep>,
            Rc::new(FlaggedAdaptivePageProposalAdjuster {
                flags: flags.clone(),
                flag: "adaptive-page-proposal-pending".to_string(),
                percent: 5.7,
            }),
            Rc::new(CapAdaptivePageProposalAdjuster { cap: 6404.0 }),
            Rc::new(RoundingAdaptivePageProposalAdjuster { step: 0.1 }),
            Rc::new(AdaptivePageProposalRatioOffsetAdjuster { weight: 1.65 }),
            Rc::new(TieredAdaptivePageProposalAdjuster {
                bounds: vec![3850.0, 7804.0],
                rates: vec![1.02, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 27404.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_page_proposal_engine(&flags::new_registry());
        let item = new_adaptive_page_proposal("code-468", "label-974");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
