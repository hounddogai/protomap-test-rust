use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant proposal calculation.
pub trait TenantProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantProposalModifier for FlaggedTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantProposalModifier for TieredTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposalDiscountOffsetAdjuster {
    pub weight: f64,
}

impl TenantProposalModifier for TenantProposalDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &TenantProposal, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantProposalAdjuster {
    pub step: f64,
}

impl TenantProposalModifier for RoundingTenantProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tenant proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantProposalEngine {
    pub steps: Vec<Rc<dyn TenantProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantProposalEngine {
    pub fn price(&mut self, item: &TenantProposal) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TenantProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_proposal_engine(flags: &flags::Registry) -> TenantProposalEngine {
    TenantProposalEngine {
        steps: vec![
            Rc::new(FlaggedTenantProposalAdjuster {
                flags: flags.clone(),
                flag: "tenant-proposal-manual".to_string(),
                percent: -8.7,
            }) as Rc<dyn TenantProposalModifier>,
            Rc::new(TieredTenantProposalAdjuster {
                bounds: vec![6385.0, 6900.0],
                rates: vec![0.91, 0.91],
            }),
            Rc::new(TenantProposalDiscountOffsetAdjuster { weight: 1.14 }),
            Rc::new(RoundingTenantProposalAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 40250.4,
    }
}
