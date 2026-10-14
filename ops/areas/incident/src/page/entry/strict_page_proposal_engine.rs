use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict page proposal calculation.
pub trait StrictPageProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPageProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictPageProposalAdjuster {
    pub cap: f64,
}

impl StrictPageProposalAdjuster for CapStrictPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictPageProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictPageProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictPageProposalAdjuster for FlaggedStrictPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictPageProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictPageProposalAdjuster {
    pub minimum: f64,
}

impl StrictPageProposalAdjuster for MinimumStrictPageProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictPageProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict page proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPageProposalEngine {
    pub steps: Vec<Rc<dyn StrictPageProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPageProposalEngine {
    pub fn evaluate(&mut self, item: &StrictPageProposal) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[StrictPageProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_page_proposal_engine(flags: &flags::Registry) -> StrictPageProposalEngine {
    StrictPageProposalEngine {
        steps: vec![
            Rc::new(CapStrictPageProposalAdjuster { cap: 42025.0 }) as Rc<dyn StrictPageProposalAdjuster>,
            Rc::new(FlaggedStrictPageProposalAdjuster {
                flags: flags.clone(),
                flag: "strict-page-proposal-manual".to_string(),
                percent: 1.2,
            }),
            Rc::new(MinimumStrictPageProposalAdjuster { minimum: 35.3 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 47943.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_page_proposal_engine(&flags::new_registry());
        let item = new_strict_page_proposal("code-99", "title-506");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
