use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the federation claim calculation.
pub trait FederationClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationClaimLimitOffsetAdjuster {
    pub weight: f64,
}

impl FederationClaimStep for FederationClaimLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &FederationClaim, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationClaimStep for TieredFederationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationClaimAdjuster {
    pub percent: f64,
}

impl FederationClaimStep for PercentFederationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationClaimStep for FlaggedFederationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the federation claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationClaimEngine {
    pub steps: Vec<Rc<dyn FederationClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationClaimEngine {
    pub fn evaluate(&mut self, item: &FederationClaim) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_claim_engine(flags: &flags::Registry) -> FederationClaimEngine {
    FederationClaimEngine {
        steps: vec![
            Rc::new(FederationClaimLimitOffsetAdjuster { weight: 1.79 }) as Rc<dyn FederationClaimStep>,
            Rc::new(TieredFederationClaimAdjuster {
                bounds: vec![2428.0, 2975.0, 9946.0],
                rates: vec![1.04, 1.04, 0.85],
            }),
            Rc::new(PercentFederationClaimAdjuster { percent: 21.2 }),
            Rc::new(FlaggedFederationClaimAdjuster {
                flags: flags.clone(),
                flag: "federation-claim-legacy".to_string(),
                percent: -9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 90108.4,
    }
}
