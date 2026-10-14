use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze proposal calculation.
pub trait FreezeProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeProposalAdjuster {
    pub minimum: f64,
}

impl FreezeProposalFactor for MinimumFreezeProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFreezeProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeProposalFactor for SourceFreezeProposalAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &FreezeProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeProposalAdjuster {
    pub step: f64,
}

impl FreezeProposalFactor for RoundingFreezeProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the freeze proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeProposalEngine {
    pub steps: Vec<Rc<dyn FreezeProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeProposalEngine {
    pub fn run(&mut self, item: &FreezeProposal) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeProposalFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[FreezeProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_proposal_engine() -> FreezeProposalEngine {
    FreezeProposalEngine {
        steps: vec![
            Rc::new(MinimumFreezeProposalAdjuster { minimum: 43.8 }) as Rc<dyn FreezeProposalFactor>,
            Rc::new(SourceFreezeProposalAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.02),
                    ("south".to_string(), 0.82),
                    ("online".to_string(), 1.13),
                ]),
            }),
            Rc::new(RoundingFreezeProposalAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 89765.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_proposal_engine();
        let item = new_freeze_proposal("code-81", "label-297");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
