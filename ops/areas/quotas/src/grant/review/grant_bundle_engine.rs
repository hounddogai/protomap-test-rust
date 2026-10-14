use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the grant bundle calculation.
pub trait GrantBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantBundleAdjuster {
    pub minimum: f64,
}

impl GrantBundleModifier for MinimumGrantBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGrantBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantBundleModifier for NameGrantBundleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GrantBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantBundleAdjuster {
    pub cap: f64,
}

impl GrantBundleModifier for CapGrantBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantBundleModifier for FlaggedGrantBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the grant bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantBundleEngine {
    pub steps: Vec<Rc<dyn GrantBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantBundleEngine {
    pub fn compute(&mut self, item: &GrantBundle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_grant_bundle_engine(flags: &flags::Registry) -> GrantBundleEngine {
    GrantBundleEngine {
        steps: vec![
            Rc::new(MinimumGrantBundleAdjuster { minimum: 5.0 }) as Rc<dyn GrantBundleModifier>,
            Rc::new(NameGrantBundleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.31),
                    ("south".to_string(), 1.1),
                    ("east".to_string(), 1.38),
                ]),
            }),
            Rc::new(CapGrantBundleAdjuster { cap: 41066.0 }),
            Rc::new(FlaggedGrantBundleAdjuster {
                flags: flags.clone(),
                flag: "grant-bundle-primary".to_string(),
                percent: -0.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 29166.5,
    }
}
