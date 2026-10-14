use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling usage manifest calculation.
pub trait RollingUsageManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingUsageManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingUsageManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingUsageManifestStep for TieredRollingUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingUsageManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingUsageManifestAdjuster {
    pub cap: f64,
}

impl RollingUsageManifestStep for CapRollingUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingUsageManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingUsageManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingUsageManifestStep for FlaggedRollingUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingUsageManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingUsageManifestAdjuster {
    pub step: f64,
}

impl RollingUsageManifestStep for RoundingRollingUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingUsageManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRollingUsageManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingUsageManifestStep for CountryRollingUsageManifestAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RollingUsageManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingUsageManifestAdjuster {
    pub minimum: f64,
}

impl RollingUsageManifestStep for MinimumRollingUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingUsageManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling usage manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingUsageManifestEngine {
    pub steps: Vec<Rc<dyn RollingUsageManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingUsageManifestEngine {
    pub fn compute(&mut self, item: &RollingUsageManifest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingUsageManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingUsageManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_usage_manifest_engine(flags: &flags::Registry) -> RollingUsageManifestEngine {
    RollingUsageManifestEngine {
        steps: vec![
            Rc::new(TieredRollingUsageManifestAdjuster {
                bounds: vec![2687.0, 7034.0],
                rates: vec![0.98, 1.06],
            }) as Rc<dyn RollingUsageManifestStep>,
            Rc::new(CapRollingUsageManifestAdjuster { cap: 48434.0 }),
            Rc::new(FlaggedRollingUsageManifestAdjuster {
                flags: flags.clone(),
                flag: "rolling-usage-manifest-batched".to_string(),
                percent: 1.9,
            }),
            Rc::new(RoundingRollingUsageManifestAdjuster { step: 1.0 }),
            Rc::new(CountryRollingUsageManifestAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.28),
                    ("central".to_string(), 1.01),
                    ("online".to_string(), 1.38),
                ]),
            }),
            Rc::new(MinimumRollingUsageManifestAdjuster { minimum: 16.1 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 32493.1,
    }
}
