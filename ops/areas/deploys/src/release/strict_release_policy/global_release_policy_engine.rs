use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global release policy calculation.
pub trait GlobalReleasePolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReleasePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicyMinimumOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReleasePolicyAdjuster for GlobalReleasePolicyMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReleasePolicy, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReleasePolicyAdjuster {
    pub minimum: f64,
}

impl GlobalReleasePolicyAdjuster for MinimumGlobalReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReleasePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalReleasePolicyAdjuster {
    pub cap: f64,
}

impl GlobalReleasePolicyAdjuster for CapGlobalReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalReleasePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionGlobalReleasePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalReleasePolicyAdjuster for RegionGlobalReleasePolicyAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &GlobalReleasePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalReleasePolicyAdjuster {
    pub percent: f64,
}

impl GlobalReleasePolicyAdjuster for PercentGlobalReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalReleasePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalReleasePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalReleasePolicyAdjuster for FlaggedGlobalReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalReleasePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global release policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReleasePolicyEngine {
    pub steps: Vec<Rc<dyn GlobalReleasePolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReleasePolicyEngine {
    pub fn price(&mut self, item: &GlobalReleasePolicy) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReleasePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_release_policy_engine(flags: &flags::Registry) -> GlobalReleasePolicyEngine {
    GlobalReleasePolicyEngine {
        steps: vec![
            Rc::new(GlobalReleasePolicyMinimumOffsetAdjuster { weight: 0.73 }) as Rc<dyn GlobalReleasePolicyAdjuster>,
            Rc::new(MinimumGlobalReleasePolicyAdjuster { minimum: 26.2 }),
            Rc::new(CapGlobalReleasePolicyAdjuster { cap: 16885.0 }),
            Rc::new(RegionGlobalReleasePolicyAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.85), ("south".to_string(), 0.93)]),
            }),
            Rc::new(PercentGlobalReleasePolicyAdjuster { percent: -6.7 }),
            Rc::new(FlaggedGlobalReleasePolicyAdjuster {
                flags: flags.clone(),
                flag: "global-release-policy-strict".to_string(),
                percent: 3.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 20571.7,
    }
}
