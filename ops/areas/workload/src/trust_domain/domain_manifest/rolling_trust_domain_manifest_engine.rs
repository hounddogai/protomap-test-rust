use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling trust domain manifest calculation.
pub trait RollingTrustDomainManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTrustDomainManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTrustDomainManifestAdjuster {
    pub percent: f64,
}

impl RollingTrustDomainManifestAdjuster for PercentRollingTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTrustDomainManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingTrustDomainManifestAdjuster {
    pub step: f64,
}

impl RollingTrustDomainManifestAdjuster for RoundingRollingTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingTrustDomainManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTrustDomainManifestAdjuster {
    pub minimum: f64,
}

impl RollingTrustDomainManifestAdjuster for MinimumRollingTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTrustDomainManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTrustDomainManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTrustDomainManifestAdjuster for FlaggedRollingTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTrustDomainManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTrustDomainManifestMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RollingTrustDomainManifestAdjuster for RollingTrustDomainManifestMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTrustDomainManifest, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTrustDomainManifestAdjuster {
    pub cap: f64,
}

impl RollingTrustDomainManifestAdjuster for CapRollingTrustDomainManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTrustDomainManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling trust domain manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTrustDomainManifestEngine {
    pub steps: Vec<Rc<dyn RollingTrustDomainManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTrustDomainManifestEngine {
    pub fn price(&mut self, item: &RollingTrustDomainManifest) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingTrustDomainManifestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingTrustDomainManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_trust_domain_manifest_engine(flags: &flags::Registry) -> RollingTrustDomainManifestEngine {
    RollingTrustDomainManifestEngine {
        steps: vec![
            Rc::new(PercentRollingTrustDomainManifestAdjuster { percent: -1.3 })
                as Rc<dyn RollingTrustDomainManifestAdjuster>,
            Rc::new(RoundingRollingTrustDomainManifestAdjuster { step: 0.25 }),
            Rc::new(MinimumRollingTrustDomainManifestAdjuster { minimum: 8.8 }),
            Rc::new(FlaggedRollingTrustDomainManifestAdjuster {
                flags: flags.clone(),
                flag: "rolling-trust-domain-manifest-legacy".to_string(),
                percent: 8.6,
            }),
            Rc::new(RollingTrustDomainManifestMaximumOffsetAdjuster { weight: 0.44 }),
            Rc::new(CapRollingTrustDomainManifestAdjuster { cap: 19696.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 67994.6,
    }
}
