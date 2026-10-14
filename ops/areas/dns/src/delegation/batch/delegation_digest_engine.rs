use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation digest calculation.
pub trait DelegationDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationDigest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationDigestFactor for FlaggedDelegationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDigestRateOffsetAdjuster {
    pub weight: f64,
}

impl DelegationDigestFactor for DelegationDigestRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationDigest, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationDigestAdjuster {
    pub cap: f64,
}

impl DelegationDigestFactor for CapDelegationDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationDigestAdjuster {
    pub step: f64,
}

impl DelegationDigestFactor for RoundingDelegationDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the delegation digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationDigestEngine {
    pub steps: Vec<Rc<dyn DelegationDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationDigestEngine {
    pub fn compute(&mut self, item: &DelegationDigest) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[DelegationDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_digest_engine(flags: &flags::Registry) -> DelegationDigestEngine {
    DelegationDigestEngine {
        steps: vec![
            Rc::new(FlaggedDelegationDigestAdjuster {
                flags: flags.clone(),
                flag: "delegation-digest-weighted".to_string(),
                percent: -4.8,
            }) as Rc<dyn DelegationDigestFactor>,
            Rc::new(DelegationDigestRateOffsetAdjuster { weight: 1.66 }),
            Rc::new(CapDelegationDigestAdjuster { cap: 29967.0 }),
            Rc::new(RoundingDelegationDigestAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 55673.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_digest_engine(&flags::new_registry());
        let item = new_delegation_digest("code-737", "title-357");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
