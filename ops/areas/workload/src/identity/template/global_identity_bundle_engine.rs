use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global identity bundle calculation.
pub trait GlobalIdentityBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalIdentityBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalIdentityBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalIdentityBundleAdjuster for FlaggedGlobalIdentityBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalIdentityBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionGlobalIdentityBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalIdentityBundleAdjuster for RegionGlobalIdentityBundleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &GlobalIdentityBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalIdentityBundleAdjuster {
    pub percent: f64,
}

impl GlobalIdentityBundleAdjuster for PercentGlobalIdentityBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalIdentityBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalIdentityBundleAdjuster {
    pub cap: f64,
}

impl GlobalIdentityBundleAdjuster for CapGlobalIdentityBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalIdentityBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalIdentityBundleAdjuster {
    pub minimum: f64,
}

impl GlobalIdentityBundleAdjuster for MinimumGlobalIdentityBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalIdentityBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global identity bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalIdentityBundleEngine {
    pub steps: Vec<Rc<dyn GlobalIdentityBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalIdentityBundleEngine {
    pub fn compute(&mut self, item: &GlobalIdentityBundle) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalIdentityBundleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalIdentityBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_identity_bundle_engine(flags: &flags::Registry) -> GlobalIdentityBundleEngine {
    GlobalIdentityBundleEngine {
        steps: vec![
            Rc::new(FlaggedGlobalIdentityBundleAdjuster {
                flags: flags.clone(),
                flag: "global-identity-bundle-pending".to_string(),
                percent: -5.4,
            }) as Rc<dyn GlobalIdentityBundleAdjuster>,
            Rc::new(RegionGlobalIdentityBundleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.73), ("online".to_string(), 0.9)]),
            }),
            Rc::new(PercentGlobalIdentityBundleAdjuster { percent: -17.1 }),
            Rc::new(CapGlobalIdentityBundleAdjuster { cap: 1845.0 }),
            Rc::new(MinimumGlobalIdentityBundleAdjuster { minimum: 47.5 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 10245.2,
    }
}
