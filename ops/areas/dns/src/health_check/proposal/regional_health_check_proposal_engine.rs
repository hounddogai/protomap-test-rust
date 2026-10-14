use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional health check proposal calculation.
pub trait RegionalHealthCheckProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalHealthCheckProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalHealthCheckProposalAdjuster {
    pub step: f64,
}

impl RegionalHealthCheckProposalFactor for RoundingRegionalHealthCheckProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalHealthCheckProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalHealthCheckProposalAdjuster {
    pub minimum: f64,
}

impl RegionalHealthCheckProposalFactor for MinimumRegionalHealthCheckProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalHealthCheckProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalHealthCheckProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalHealthCheckProposalFactor for TieredRegionalHealthCheckProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalHealthCheckProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalHealthCheckProposalAdjuster {
    pub percent: f64,
}

impl RegionalHealthCheckProposalFactor for PercentRegionalHealthCheckProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalHealthCheckProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional health check proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalHealthCheckProposalEngine {
    pub steps: Vec<Rc<dyn RegionalHealthCheckProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalHealthCheckProposalEngine {
    pub fn compute(&mut self, item: &RegionalHealthCheckProposal) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalHealthCheckProposalFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalHealthCheckProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_health_check_proposal_engine() -> RegionalHealthCheckProposalEngine {
    RegionalHealthCheckProposalEngine {
        steps: vec![
            Rc::new(RoundingRegionalHealthCheckProposalAdjuster { step: 0.1 })
                as Rc<dyn RegionalHealthCheckProposalFactor>,
            Rc::new(MinimumRegionalHealthCheckProposalAdjuster { minimum: 32.9 }),
            Rc::new(TieredRegionalHealthCheckProposalAdjuster {
                bounds: vec![7213.0, 9125.0, 9198.0],
                rates: vec![0.88, 1.06, 1.19],
            }),
            Rc::new(PercentRegionalHealthCheckProposalAdjuster { percent: -24.5 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 67025.1,
    }
}
