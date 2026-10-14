use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant digest calculation.
pub trait GrantDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantDigestAdjuster {
    pub minimum: f64,
}

impl GrantDigestStep for MinimumGrantDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantDigestStep for FlaggedGrantDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleGrantDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantDigestStep for TitleGrantDigestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &GrantDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantDigestAdjuster {
    pub percent: f64,
}

impl GrantDigestStep for PercentGrantDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantDigestRevisionOffsetAdjuster {
    pub weight: f64,
}

impl GrantDigestStep for GrantDigestRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &GrantDigest, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the grant digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantDigestEngine {
    pub steps: Vec<Rc<dyn GrantDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantDigestEngine {
    pub fn price(&mut self, item: &GrantDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GrantDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_digest_engine(flags: &flags::Registry) -> GrantDigestEngine {
    GrantDigestEngine {
        steps: vec![
            Rc::new(MinimumGrantDigestAdjuster { minimum: 22.9 }) as Rc<dyn GrantDigestStep>,
            Rc::new(FlaggedGrantDigestAdjuster {
                flags: flags.clone(),
                flag: "grant-digest-fallback".to_string(),
                percent: 3.7,
            }),
            Rc::new(TitleGrantDigestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.99)]),
            }),
            Rc::new(PercentGrantDigestAdjuster { percent: -17.1 }),
            Rc::new(GrantDigestRevisionOffsetAdjuster { weight: 1.75 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 97707.5,
    }
}
