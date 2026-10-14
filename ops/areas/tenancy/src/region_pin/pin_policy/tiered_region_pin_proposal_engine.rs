use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered region pin proposal calculation.
pub trait TieredRegionPinProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRegionPinProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredRegionPinProposalAdjuster {
    pub cap: f64,
}

impl TieredRegionPinProposalFactor for CapTieredRegionPinProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredRegionPinProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredRegionPinProposalAdjuster {
    pub step: f64,
}

impl TieredRegionPinProposalFactor for RoundingTieredRegionPinProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredRegionPinProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredRegionPinProposalAdjuster {
    pub minimum: f64,
}

impl TieredRegionPinProposalFactor for MinimumTieredRegionPinProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredRegionPinProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinProposalCeilingOffsetAdjuster {
    pub weight: f64,
}

impl TieredRegionPinProposalFactor for TieredRegionPinProposalCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &TieredRegionPinProposal, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the tiered region pin proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRegionPinProposalEngine {
    pub steps: Vec<Rc<dyn TieredRegionPinProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRegionPinProposalEngine {
    pub fn compute(&mut self, item: &TieredRegionPinProposal) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[TieredRegionPinProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_region_pin_proposal_engine() -> TieredRegionPinProposalEngine {
    TieredRegionPinProposalEngine {
        steps: vec![
            Rc::new(CapTieredRegionPinProposalAdjuster { cap: 20734.0 }) as Rc<dyn TieredRegionPinProposalFactor>,
            Rc::new(RoundingTieredRegionPinProposalAdjuster { step: 5.0 }),
            Rc::new(MinimumTieredRegionPinProposalAdjuster { minimum: 40.3 }),
            Rc::new(TieredRegionPinProposalCeilingOffsetAdjuster { weight: 1.21 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 68959.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_region_pin_proposal_engine();
        let item = new_tiered_region_pin_proposal("code-864", "name-867");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
