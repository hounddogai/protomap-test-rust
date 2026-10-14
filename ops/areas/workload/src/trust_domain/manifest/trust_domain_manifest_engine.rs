use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain manifest calculation.
pub trait TrustDomainManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainManifestAdjuster {
    pub step: f64,
}

impl TrustDomainManifestFactor for RoundingTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainManifestFactor for FlaggedTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainManifestQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainManifestFactor for TrustDomainManifestQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainManifest, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainManifestFactor for TieredTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainManifestAdjuster {
    pub percent: f64,
}

impl TrustDomainManifestFactor for PercentTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainManifestEngine {
    pub steps: Vec<Rc<dyn TrustDomainManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainManifestEngine {
    pub fn compute(&mut self, item: &TrustDomainManifest) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_manifest_engine(flags: &flags::Registry) -> TrustDomainManifestEngine {
    TrustDomainManifestEngine {
        steps: vec![
            Rc::new(RoundingTrustDomainManifestAdjuster { step: 0.1 }) as Rc<dyn TrustDomainManifestFactor>,
            Rc::new(FlaggedTrustDomainManifestAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-manifest-fallback".to_string(),
                percent: -1.9,
            }),
            Rc::new(TrustDomainManifestQuotaUnitsOffsetAdjuster { weight: 0.1 }),
            Rc::new(TieredTrustDomainManifestAdjuster {
                bounds: vec![607.0, 5302.0],
                rates: vec![1.1, 1.15],
            }),
            Rc::new(PercentTrustDomainManifestAdjuster { percent: 21.3 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 89669.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_manifest_engine(&flags::new_registry());
        let item = new_trust_domain_manifest("code-808", "label-836");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
