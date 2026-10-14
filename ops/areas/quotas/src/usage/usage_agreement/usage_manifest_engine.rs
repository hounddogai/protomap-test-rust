use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the usage manifest calculation.
pub trait UsageManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageManifestAdjuster {
    pub minimum: f64,
}

impl UsageManifestModifier for MinimumUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageManifestUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl UsageManifestModifier for UsageManifestUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &UsageManifest, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageManifestAdjuster {
    pub cap: f64,
}

impl UsageManifestModifier for CapUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageManifestModifier for FlaggedUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageManifestModifier for TieredUsageManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryUsageManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageManifestModifier for CountryUsageManifestAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &UsageManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the usage manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageManifestEngine {
    pub steps: Vec<Rc<dyn UsageManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageManifestEngine {
    pub fn compute(&mut self, item: &UsageManifest) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageManifestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_usage_manifest_engine(flags: &flags::Registry) -> UsageManifestEngine {
    UsageManifestEngine {
        steps: vec![
            Rc::new(MinimumUsageManifestAdjuster { minimum: 19.4 }) as Rc<dyn UsageManifestModifier>,
            Rc::new(UsageManifestUpdatedAtOffsetAdjuster { weight: 0.3 }),
            Rc::new(CapUsageManifestAdjuster { cap: 17011.0 }),
            Rc::new(FlaggedUsageManifestAdjuster {
                flags: flags.clone(),
                flag: "usage-manifest-fallback".to_string(),
                percent: 5.7,
            }),
            Rc::new(TieredUsageManifestAdjuster {
                bounds: vec![74.0, 2420.0, 4496.0],
                rates: vec![0.95, 0.93, 1.09],
            }),
            Rc::new(CountryUsageManifestAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.75),
                    ("north".to_string(), 1.25),
                    ("south".to_string(), 0.87),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 55007.8,
    }
}
