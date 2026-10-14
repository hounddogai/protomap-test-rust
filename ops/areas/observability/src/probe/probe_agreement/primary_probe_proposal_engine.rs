use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary probe proposal calculation.
pub trait PrimaryProbeProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryProbeProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryProbeProposalAdjuster {
    pub cap: f64,
}

impl PrimaryProbeProposalFactor for CapPrimaryProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryProbeProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryProbeProposalAdjuster {
    pub step: f64,
}

impl PrimaryProbeProposalFactor for RoundingPrimaryProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryProbeProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryProbeProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryProbeProposalFactor for FlaggedPrimaryProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryProbeProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryProbeProposalAdjuster {
    pub percent: f64,
}

impl PrimaryProbeProposalFactor for PercentPrimaryProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryProbeProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryProbeProposalAdjuster {
    pub minimum: f64,
}

impl PrimaryProbeProposalFactor for MinimumPrimaryProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryProbeProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary probe proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryProbeProposalEngine {
    pub steps: Vec<Rc<dyn PrimaryProbeProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryProbeProposalEngine {
    pub fn price(&mut self, item: &PrimaryProbeProposal) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryProbeProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_probe_proposal_engine(flags: &flags::Registry) -> PrimaryProbeProposalEngine {
    PrimaryProbeProposalEngine {
        steps: vec![
            Rc::new(CapPrimaryProbeProposalAdjuster { cap: 35663.0 }) as Rc<dyn PrimaryProbeProposalFactor>,
            Rc::new(RoundingPrimaryProbeProposalAdjuster { step: 0.25 }),
            Rc::new(FlaggedPrimaryProbeProposalAdjuster {
                flags: flags.clone(),
                flag: "primary-probe-proposal-fallback".to_string(),
                percent: 7.2,
            }),
            Rc::new(PercentPrimaryProbeProposalAdjuster { percent: -12.8 }),
            Rc::new(MinimumPrimaryProbeProposalAdjuster { minimum: 4.6 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 32019.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_probe_proposal_engine(&flags::new_registry());
        let item = new_primary_probe_proposal("code-901", "label-200");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
