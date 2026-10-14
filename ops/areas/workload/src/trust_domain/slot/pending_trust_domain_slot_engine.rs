use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending trust domain slot calculation.
pub trait PendingTrustDomainSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTrustDomainSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingTrustDomainSlotAdjuster {
    pub percent: f64,
}

impl PendingTrustDomainSlotFactor for PercentPendingTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingTrustDomainSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainSlotLimitOffsetAdjuster {
    pub weight: f64,
}

impl PendingTrustDomainSlotFactor for PendingTrustDomainSlotLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &PendingTrustDomainSlot, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingTrustDomainSlotAdjuster {
    pub cap: f64,
}

impl PendingTrustDomainSlotFactor for CapPendingTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingTrustDomainSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingTrustDomainSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingTrustDomainSlotFactor for TieredPendingTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingTrustDomainSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingTrustDomainSlotAdjuster {
    pub step: f64,
}

impl PendingTrustDomainSlotFactor for RoundingPendingTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingTrustDomainSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingTrustDomainSlotAdjuster {
    pub minimum: f64,
}

impl PendingTrustDomainSlotFactor for MinimumPendingTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingTrustDomainSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending trust domain slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTrustDomainSlotEngine {
    pub steps: Vec<Rc<dyn PendingTrustDomainSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTrustDomainSlotEngine {
    pub fn compute(&mut self, item: &PendingTrustDomainSlot) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingTrustDomainSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_trust_domain_slot_engine() -> PendingTrustDomainSlotEngine {
    PendingTrustDomainSlotEngine {
        steps: vec![
            Rc::new(PercentPendingTrustDomainSlotAdjuster { percent: -5.5 }) as Rc<dyn PendingTrustDomainSlotFactor>,
            Rc::new(PendingTrustDomainSlotLimitOffsetAdjuster { weight: 0.28 }),
            Rc::new(CapPendingTrustDomainSlotAdjuster { cap: 45142.0 }),
            Rc::new(TieredPendingTrustDomainSlotAdjuster {
                bounds: vec![2084.0, 7069.0],
                rates: vec![0.87, 1.07],
            }),
            Rc::new(RoundingPendingTrustDomainSlotAdjuster { step: 0.1 }),
            Rc::new(MinimumPendingTrustDomainSlotAdjuster { minimum: 0.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 74399.9,
    }
}
