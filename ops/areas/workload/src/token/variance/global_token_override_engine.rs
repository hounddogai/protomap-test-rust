use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global token override calculation.
pub trait GlobalTokenOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalTokenOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalTokenOverrideAdjuster {
    pub percent: f64,
}

impl GlobalTokenOverrideAdjuster for PercentGlobalTokenOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalTokenOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalTokenOverrideAdjuster {
    pub cap: f64,
}

impl GlobalTokenOverrideAdjuster for CapGlobalTokenOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalTokenOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalTokenOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalTokenOverrideAdjuster for FlaggedGlobalTokenOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalTokenOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterGlobalTokenOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalTokenOverrideAdjuster for ClusterGlobalTokenOverrideAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &GlobalTokenOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverrideQuantityOffsetAdjuster {
    pub weight: f64,
}

impl GlobalTokenOverrideAdjuster for GlobalTokenOverrideQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalTokenOverride, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalTokenOverrideAdjuster {
    pub step: f64,
}

impl GlobalTokenOverrideAdjuster for RoundingGlobalTokenOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalTokenOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global token override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalTokenOverrideEngine {
    pub steps: Vec<Rc<dyn GlobalTokenOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalTokenOverrideEngine {
    pub fn price(&mut self, item: &GlobalTokenOverride) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[GlobalTokenOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_token_override_engine(flags: &flags::Registry) -> GlobalTokenOverrideEngine {
    GlobalTokenOverrideEngine {
        steps: vec![
            Rc::new(PercentGlobalTokenOverrideAdjuster { percent: 21.5 }) as Rc<dyn GlobalTokenOverrideAdjuster>,
            Rc::new(CapGlobalTokenOverrideAdjuster { cap: 10561.0 }),
            Rc::new(FlaggedGlobalTokenOverrideAdjuster {
                flags: flags.clone(),
                flag: "global-token-override-batched".to_string(),
                percent: 8.2,
            }),
            Rc::new(ClusterGlobalTokenOverrideAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.39), ("north".to_string(), 1.38)]),
            }),
            Rc::new(GlobalTokenOverrideQuantityOffsetAdjuster { weight: 1.49 }),
            Rc::new(RoundingGlobalTokenOverrideAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 45568.8,
    }
}
