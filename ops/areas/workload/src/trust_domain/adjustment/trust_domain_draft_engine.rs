use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the trust domain draft calculation.
pub trait TrustDomainDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainDraftAdjuster {
    pub cap: f64,
}

impl TrustDomainDraftFactor for CapTrustDomainDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTrustDomainDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainDraftFactor for RegionTrustDomainDraftAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TrustDomainDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainDraftAdjuster {
    pub percent: f64,
}

impl TrustDomainDraftFactor for PercentTrustDomainDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDraftPriorityOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainDraftFactor for TrustDomainDraftPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainDraft, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainDraftAdjuster {
    pub step: f64,
}

impl TrustDomainDraftFactor for RoundingTrustDomainDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainDraftFactor for TieredTrustDomainDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the trust domain draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainDraftEngine {
    pub steps: Vec<Rc<dyn TrustDomainDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainDraftEngine {
    pub fn price(&mut self, item: &TrustDomainDraft) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_trust_domain_draft_engine() -> TrustDomainDraftEngine {
    TrustDomainDraftEngine {
        steps: vec![
            Rc::new(CapTrustDomainDraftAdjuster { cap: 4533.0 }) as Rc<dyn TrustDomainDraftFactor>,
            Rc::new(RegionTrustDomainDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.39), ("north".to_string(), 0.78)]),
            }),
            Rc::new(PercentTrustDomainDraftAdjuster { percent: -13.8 }),
            Rc::new(TrustDomainDraftPriorityOffsetAdjuster { weight: 0.94 }),
            Rc::new(RoundingTrustDomainDraftAdjuster { step: 0.25 }),
            Rc::new(TieredTrustDomainDraftAdjuster {
                bounds: vec![1490.0, 1761.0, 7829.0, 9074.0],
                rates: vec![1.11, 0.82, 1.11, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 4768.4,
    }
}
