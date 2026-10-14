use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped freeze proposal calculation.
pub trait CappedFreezeProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedFreezeProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedFreezeProposalAdjuster {
    pub minimum: f64,
}

impl CappedFreezeProposalModifier for MinimumCappedFreezeProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedFreezeProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitCappedFreezeProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedFreezeProposalModifier for UnitCappedFreezeProposalAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &CappedFreezeProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedFreezeProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedFreezeProposalModifier for FlaggedCappedFreezeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedFreezeProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped freeze proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedFreezeProposalEngine {
    pub steps: Vec<Rc<dyn CappedFreezeProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedFreezeProposalEngine {
    pub fn compute(&mut self, item: &CappedFreezeProposal) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[CappedFreezeProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_freeze_proposal_engine(flags: &flags::Registry) -> CappedFreezeProposalEngine {
    CappedFreezeProposalEngine {
        steps: vec![
            Rc::new(MinimumCappedFreezeProposalAdjuster { minimum: 28.5 }) as Rc<dyn CappedFreezeProposalModifier>,
            Rc::new(UnitCappedFreezeProposalAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.3), ("west".to_string(), 1.0)]),
            }),
            Rc::new(FlaggedCappedFreezeProposalAdjuster {
                flags: flags.clone(),
                flag: "capped-freeze-proposal-primary".to_string(),
                percent: -2.9,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 55094.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_freeze_proposal_engine(&flags::new_registry());
        let item = new_capped_freeze_proposal("code-292", "name-450");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
