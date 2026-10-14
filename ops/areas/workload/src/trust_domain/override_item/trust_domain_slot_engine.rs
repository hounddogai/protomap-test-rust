use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the trust domain slot calculation.
pub trait TrustDomainSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainSlotAdjuster {
    pub cap: f64,
}

impl TrustDomainSlotAdjuster for CapTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainSlotAdjuster for TieredTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainSlotAdjuster for FlaggedTrustDomainSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainSlotEngine {
    pub steps: Vec<Rc<dyn TrustDomainSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainSlotEngine {
    pub fn evaluate(&mut self, item: &TrustDomainSlot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainSlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_trust_domain_slot_engine(flags: &flags::Registry) -> TrustDomainSlotEngine {
    TrustDomainSlotEngine {
        steps: vec![
            Rc::new(CapTrustDomainSlotAdjuster { cap: 7335.0 }) as Rc<dyn TrustDomainSlotAdjuster>,
            Rc::new(TieredTrustDomainSlotAdjuster {
                bounds: vec![1919.0, 5090.0, 5243.0, 5848.0],
                rates: vec![1.06, 0.96, 1.05, 0.98],
            }),
            Rc::new(FlaggedTrustDomainSlotAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-slot-scheduled".to_string(),
                percent: 9.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 5063.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_slot_engine(&flags::new_registry());
        let item = new_trust_domain_slot("code-80", "label-735");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
