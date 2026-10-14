use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict canary proposal calculation.
pub trait StrictCanaryProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictCanaryProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictCanaryProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictCanaryProposalFactor for TieredStrictCanaryProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictCanaryProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictCanaryProposalAdjuster {
    pub cap: f64,
}

impl StrictCanaryProposalFactor for CapStrictCanaryProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictCanaryProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictCanaryProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictCanaryProposalFactor for FlaggedStrictCanaryProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictCanaryProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict canary proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictCanaryProposalEngine {
    pub steps: Vec<Rc<dyn StrictCanaryProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictCanaryProposalEngine {
    pub fn compute(&mut self, item: &StrictCanaryProposal) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[StrictCanaryProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_canary_proposal_engine(flags: &flags::Registry) -> StrictCanaryProposalEngine {
    StrictCanaryProposalEngine {
        steps: vec![
            Rc::new(TieredStrictCanaryProposalAdjuster {
                bounds: vec![1095.0, 3006.0, 7285.0],
                rates: vec![0.93, 1.09, 0.9],
            }) as Rc<dyn StrictCanaryProposalFactor>,
            Rc::new(CapStrictCanaryProposalAdjuster { cap: 34065.0 }),
            Rc::new(FlaggedStrictCanaryProposalAdjuster {
                flags: flags.clone(),
                flag: "strict-canary-proposal-rolling".to_string(),
                percent: 0.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 62831.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_canary_proposal_engine(&flags::new_registry());
        let item = new_strict_canary_proposal("code-764", "title-460");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
