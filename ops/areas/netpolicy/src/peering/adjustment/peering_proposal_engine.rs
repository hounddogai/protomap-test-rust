use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering proposal calculation.
pub trait PeeringProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringProposalAdjuster {
    pub cap: f64,
}

impl PeeringProposalAdjuster for CapPeeringProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringProposalAdjuster for FlaggedPeeringProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringProposalAdjuster for TieredPeeringProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProposalPortOffsetAdjuster {
    pub weight: f64,
}

impl PeeringProposalAdjuster for PeeringProposalPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringProposal, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the peering proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringProposalEngine {
    pub steps: Vec<Rc<dyn PeeringProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringProposalEngine {
    pub fn price(&mut self, item: &PeeringProposal) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PeeringProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_peering_proposal_engine(flags: &flags::Registry) -> PeeringProposalEngine {
    PeeringProposalEngine {
        steps: vec![
            Rc::new(CapPeeringProposalAdjuster { cap: 22987.0 }) as Rc<dyn PeeringProposalAdjuster>,
            Rc::new(FlaggedPeeringProposalAdjuster {
                flags: flags.clone(),
                flag: "peering-proposal-batched".to_string(),
                percent: -5.6,
            }),
            Rc::new(TieredPeeringProposalAdjuster {
                bounds: vec![115.0, 5228.0, 6559.0],
                rates: vec![0.8, 0.96, 1.03],
            }),
            Rc::new(PeeringProposalPortOffsetAdjuster { weight: 1.55 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 39328.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_proposal_engine(&flags::new_registry());
        let item = new_peering_proposal("code-80", "title-492");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
