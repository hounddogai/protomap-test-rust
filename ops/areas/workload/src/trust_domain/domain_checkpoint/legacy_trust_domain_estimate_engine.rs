use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy trust domain estimate calculation.
pub trait LegacyTrustDomainEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyTrustDomainEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyTrustDomainEstimateAdjuster {
    pub cap: f64,
}

impl LegacyTrustDomainEstimateModifier for CapLegacyTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyTrustDomainEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyTrustDomainEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyTrustDomainEstimateModifier for FlaggedLegacyTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyTrustDomainEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyTrustDomainEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyTrustDomainEstimateModifier for TieredLegacyTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyTrustDomainEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyTrustDomainEstimateAdjuster {
    pub percent: f64,
}

impl LegacyTrustDomainEstimateModifier for PercentLegacyTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyTrustDomainEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyTrustDomainEstimateAdjuster {
    pub step: f64,
}

impl LegacyTrustDomainEstimateModifier for RoundingLegacyTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyTrustDomainEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyTrustDomainEstimateAdjuster {
    pub minimum: f64,
}

impl LegacyTrustDomainEstimateModifier for MinimumLegacyTrustDomainEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyTrustDomainEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy trust domain estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainEstimateEngine {
    pub steps: Vec<Rc<dyn LegacyTrustDomainEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyTrustDomainEstimateEngine {
    pub fn evaluate(&mut self, item: &LegacyTrustDomainEstimate) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyTrustDomainEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_trust_domain_estimate_engine(flags: &flags::Registry) -> LegacyTrustDomainEstimateEngine {
    LegacyTrustDomainEstimateEngine {
        steps: vec![
            Rc::new(CapLegacyTrustDomainEstimateAdjuster { cap: 24654.0 }) as Rc<dyn LegacyTrustDomainEstimateModifier>,
            Rc::new(FlaggedLegacyTrustDomainEstimateAdjuster {
                flags: flags.clone(),
                flag: "legacy-trust-domain-estimate-primary".to_string(),
                percent: 8.9,
            }),
            Rc::new(TieredLegacyTrustDomainEstimateAdjuster {
                bounds: vec![3326.0, 5574.0],
                rates: vec![1.09, 0.96],
            }),
            Rc::new(PercentLegacyTrustDomainEstimateAdjuster { percent: -2.9 }),
            Rc::new(RoundingLegacyTrustDomainEstimateAdjuster { step: 5.0 }),
            Rc::new(MinimumLegacyTrustDomainEstimateAdjuster { minimum: 43.5 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 81133.6,
    }
}
