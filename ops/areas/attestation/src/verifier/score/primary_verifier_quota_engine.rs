use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary verifier quota calculation.
pub trait PrimaryVerifierQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryVerifierQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPrimaryVerifierQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryVerifierQuotaModifier for ChannelPrimaryVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PrimaryVerifierQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryVerifierQuotaAdjuster {
    pub minimum: f64,
}

impl PrimaryVerifierQuotaModifier for MinimumPrimaryVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryVerifierQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryVerifierQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryVerifierQuotaModifier for TieredPrimaryVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryVerifierQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryVerifierQuotaAdjuster {
    pub step: f64,
}

impl PrimaryVerifierQuotaModifier for RoundingPrimaryVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryVerifierQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryVerifierQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryVerifierQuotaModifier for FlaggedPrimaryVerifierQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryVerifierQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary verifier quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierQuotaEngine {
    pub steps: Vec<Rc<dyn PrimaryVerifierQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryVerifierQuotaEngine {
    pub fn run(&mut self, item: &PrimaryVerifierQuota) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryVerifierQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_verifier_quota_engine(flags: &flags::Registry) -> PrimaryVerifierQuotaEngine {
    PrimaryVerifierQuotaEngine {
        steps: vec![
            Rc::new(ChannelPrimaryVerifierQuotaAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.18), ("retail".to_string(), 0.74)]),
            }) as Rc<dyn PrimaryVerifierQuotaModifier>,
            Rc::new(MinimumPrimaryVerifierQuotaAdjuster { minimum: 0.2 }),
            Rc::new(TieredPrimaryVerifierQuotaAdjuster {
                bounds: vec![3741.0, 6973.0],
                rates: vec![0.97, 1.09],
            }),
            Rc::new(RoundingPrimaryVerifierQuotaAdjuster { step: 0.05 }),
            Rc::new(FlaggedPrimaryVerifierQuotaAdjuster {
                flags: flags.clone(),
                flag: "primary-verifier-quota-strict".to_string(),
                percent: 3.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 26313.4,
    }
}
