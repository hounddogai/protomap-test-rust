use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the release digest calculation.
pub trait ReleaseDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDigestMarginOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseDigestStep for ReleaseDigestMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseDigest, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReleaseDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseDigestStep for LabelReleaseDigestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ReleaseDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseDigestStep for FlaggedReleaseDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseDigestEngine {
    pub steps: Vec<Rc<dyn ReleaseDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseDigestEngine {
    pub fn compute(&mut self, item: &ReleaseDigest) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseDigestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_digest_engine(flags: &flags::Registry) -> ReleaseDigestEngine {
    ReleaseDigestEngine {
        steps: vec![
            Rc::new(ReleaseDigestMarginOffsetAdjuster { weight: 1.83 }) as Rc<dyn ReleaseDigestStep>,
            Rc::new(LabelReleaseDigestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.09),
                    ("south".to_string(), 0.7),
                    ("retail".to_string(), 1.1),
                ]),
            }),
            Rc::new(FlaggedReleaseDigestAdjuster {
                flags: flags.clone(),
                flag: "release-digest-pending".to_string(),
                percent: -3.1,
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 58505.9,
    }
}
