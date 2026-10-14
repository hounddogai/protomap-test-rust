use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy probe proposal calculation.
pub trait LegacyProbeProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyProbeProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyProbeProposalAdjuster {
    pub step: f64,
}

impl LegacyProbeProposalFactor for RoundingLegacyProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyProbeProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyProbeProposalAdjuster {
    pub cap: f64,
}

impl LegacyProbeProposalFactor for CapLegacyProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyProbeProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyProbeProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyProbeProposalFactor for TieredLegacyProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyProbeProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyProbeProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyProbeProposalFactor for FlaggedLegacyProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyProbeProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLegacyProbeProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyProbeProposalFactor for SourceLegacyProbeProposalAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LegacyProbeProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyProbeProposalAdjuster {
    pub percent: f64,
}

impl LegacyProbeProposalFactor for PercentLegacyProbeProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyProbeProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy probe proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyProbeProposalEngine {
    pub steps: Vec<Rc<dyn LegacyProbeProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyProbeProposalEngine {
    pub fn price(&mut self, item: &LegacyProbeProposal) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[LegacyProbeProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_probe_proposal_engine(flags: &flags::Registry) -> LegacyProbeProposalEngine {
    LegacyProbeProposalEngine {
        steps: vec![
            Rc::new(RoundingLegacyProbeProposalAdjuster { step: 1.0 }) as Rc<dyn LegacyProbeProposalFactor>,
            Rc::new(CapLegacyProbeProposalAdjuster { cap: 5027.0 }),
            Rc::new(TieredLegacyProbeProposalAdjuster {
                bounds: vec![2014.0, 7558.0],
                rates: vec![0.9, 0.83],
            }),
            Rc::new(FlaggedLegacyProbeProposalAdjuster {
                flags: flags.clone(),
                flag: "legacy-probe-proposal-tiered".to_string(),
                percent: -0.1,
            }),
            Rc::new(SourceLegacyProbeProposalAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.08),
                    ("south".to_string(), 1.28),
                    ("retail".to_string(), 1.09),
                ]),
            }),
            Rc::new(PercentLegacyProbeProposalAdjuster { percent: 23.4 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 88114.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_probe_proposal_engine(&flags::new_registry());
        let item = new_legacy_probe_proposal("code-589", "title-777");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
