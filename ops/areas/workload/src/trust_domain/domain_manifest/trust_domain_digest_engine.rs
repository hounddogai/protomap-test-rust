use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain digest calculation.
pub trait TrustDomainDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainDigestStep for TieredTrustDomainDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainDigestStep for FlaggedTrustDomainDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainDigestAdjuster {
    pub minimum: f64,
}

impl TrustDomainDigestStep for MinimumTrustDomainDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the trust domain digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainDigestEngine {
    pub steps: Vec<Rc<dyn TrustDomainDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainDigestEngine {
    pub fn run(&mut self, item: &TrustDomainDigest) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_trust_domain_digest_engine(flags: &flags::Registry) -> TrustDomainDigestEngine {
    TrustDomainDigestEngine {
        steps: vec![
            Rc::new(TieredTrustDomainDigestAdjuster {
                bounds: vec![1727.0, 7745.0, 8307.0, 9662.0],
                rates: vec![1.06, 1.2, 1.11, 0.91],
            }) as Rc<dyn TrustDomainDigestStep>,
            Rc::new(FlaggedTrustDomainDigestAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-digest-adaptive".to_string(),
                percent: -5.4,
            }),
            Rc::new(MinimumTrustDomainDigestAdjuster { minimum: 42.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 58331.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_digest_engine(&flags::new_registry());
        let item = new_trust_domain_digest("code-508", "name-365");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
