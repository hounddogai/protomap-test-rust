use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe proposal calculation.
pub trait ProbeProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeProposalAdjuster {
    pub minimum: f64,
}

impl ProbeProposalModifier for MinimumProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeProposalAdjuster {
    pub step: f64,
}

impl ProbeProposalModifier for RoundingProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeProposalModifier for TieredProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeProposalModifier for FlaggedProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the probe proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeProposalEngine {
    pub steps: Vec<Rc<dyn ProbeProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeProposalEngine {
    pub fn price(&mut self, item: &ProbeProposal) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[ProbeProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_proposal_engine(flags: &flags::Registry) -> ProbeProposalEngine {
    ProbeProposalEngine {
        steps: vec![
            Rc::new(MinimumProbeProposalAdjuster { minimum: 11.7 }) as Rc<dyn ProbeProposalModifier>,
            Rc::new(RoundingProbeProposalAdjuster { step: 0.5 }),
            Rc::new(TieredProbeProposalAdjuster {
                bounds: vec![3690.0, 6047.0, 7021.0, 8287.0],
                rates: vec![1.05, 0.87, 1.12, 0.91],
            }),
            Rc::new(FlaggedProbeProposalAdjuster {
                flags: flags.clone(),
                flag: "probe-proposal-seasonal".to_string(),
                percent: 6.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 84401.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_proposal_engine(&flags::new_registry());
        let item = new_probe_proposal("code-349", "name-601");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
