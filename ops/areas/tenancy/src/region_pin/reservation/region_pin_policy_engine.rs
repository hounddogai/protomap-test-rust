use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the region pin policy calculation.
pub trait RegionPinPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinPolicyAdjuster {
    pub minimum: f64,
}

impl RegionPinPolicyFactor for MinimumRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinPolicyAdjuster {
    pub cap: f64,
}

impl RegionPinPolicyFactor for CapRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionPinPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionPinPolicyFactor for FlaggedRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionPinPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinPolicyAdjuster {
    pub step: f64,
}

impl RegionPinPolicyFactor for RoundingRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the region pin policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinPolicyEngine {
    pub steps: Vec<Rc<dyn RegionPinPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinPolicyEngine {
    pub fn run(&mut self, item: &RegionPinPolicy) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_region_pin_policy_engine(flags: &flags::Registry) -> RegionPinPolicyEngine {
    RegionPinPolicyEngine {
        steps: vec![
            Rc::new(MinimumRegionPinPolicyAdjuster { minimum: 22.5 }) as Rc<dyn RegionPinPolicyFactor>,
            Rc::new(CapRegionPinPolicyAdjuster { cap: 29466.0 }),
            Rc::new(FlaggedRegionPinPolicyAdjuster {
                flags: flags.clone(),
                flag: "region-pin-policy-seasonal".to_string(),
                percent: -8.4,
            }),
            Rc::new(RoundingRegionPinPolicyAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 58506.3,
    }
}
