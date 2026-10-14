use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional federation proposal calculation.
pub trait RegionalFederationProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFederationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalFederationProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalFederationProposalStep for FlaggedRegionalFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalFederationProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionalFederationProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalFederationProposalStep for TitleRegionalFederationProposalAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionalFederationProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalFederationProposalAdjuster {
    pub percent: f64,
}

impl RegionalFederationProposalStep for PercentRegionalFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalFederationProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalFederationProposalAdjuster {
    pub minimum: f64,
}

impl RegionalFederationProposalStep for MinimumRegionalFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalFederationProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalFederationProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalFederationProposalStep for TieredRegionalFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalFederationProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationProposalWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl RegionalFederationProposalStep for RegionalFederationProposalWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalFederationProposal, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the regional federation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFederationProposalEngine {
    pub steps: Vec<Rc<dyn RegionalFederationProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFederationProposalEngine {
    pub fn price(&mut self, item: &RegionalFederationProposal) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalFederationProposalStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFederationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_federation_proposal_engine(flags: &flags::Registry) -> RegionalFederationProposalEngine {
    RegionalFederationProposalEngine {
        steps: vec![
            Rc::new(FlaggedRegionalFederationProposalAdjuster {
                flags: flags.clone(),
                flag: "regional-federation-proposal-regional".to_string(),
                percent: 7.2,
            }) as Rc<dyn RegionalFederationProposalStep>,
            Rc::new(TitleRegionalFederationProposalAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.15), ("west".to_string(), 0.76)]),
            }),
            Rc::new(PercentRegionalFederationProposalAdjuster { percent: 12.8 }),
            Rc::new(MinimumRegionalFederationProposalAdjuster { minimum: 37.6 }),
            Rc::new(TieredRegionalFederationProposalAdjuster {
                bounds: vec![561.0, 2839.0],
                rates: vec![0.88, 0.97],
            }),
            Rc::new(RegionalFederationProposalWindowDaysOffsetAdjuster { weight: 1.44 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 95390.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_federation_proposal_engine(&flags::new_registry());
        let item = new_regional_federation_proposal("code-37", "title-432");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
