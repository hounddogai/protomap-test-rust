use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the usage profile calculation.
pub trait UsageProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageProfileAdjuster {
    pub minimum: f64,
}

impl UsageProfileAdjuster for MinimumUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageProfileAdjuster {
    pub cap: f64,
}

impl UsageProfileAdjuster for CapUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageProfileAdjuster for FlaggedUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelUsageProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageProfileAdjuster for LabelUsageProfileAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &UsageProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageProfileAdjuster {
    pub step: f64,
}

impl UsageProfileAdjuster for RoundingUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageProfileAdjuster {
    pub percent: f64,
}

impl UsageProfileAdjuster for PercentUsageProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the usage profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageProfileEngine {
    pub steps: Vec<Rc<dyn UsageProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageProfileEngine {
    pub fn run(&mut self, item: &UsageProfile) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[UsageProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_profile_engine(flags: &flags::Registry) -> UsageProfileEngine {
    UsageProfileEngine {
        steps: vec![
            Rc::new(MinimumUsageProfileAdjuster { minimum: 26.7 }) as Rc<dyn UsageProfileAdjuster>,
            Rc::new(CapUsageProfileAdjuster { cap: 19803.0 }),
            Rc::new(FlaggedUsageProfileAdjuster {
                flags: flags.clone(),
                flag: "usage-profile-manual".to_string(),
                percent: -0.6,
            }),
            Rc::new(LabelUsageProfileAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.85)]),
            }),
            Rc::new(RoundingUsageProfileAdjuster { step: 0.1 }),
            Rc::new(PercentUsageProfileAdjuster { percent: -24.7 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 89822.2,
    }
}
