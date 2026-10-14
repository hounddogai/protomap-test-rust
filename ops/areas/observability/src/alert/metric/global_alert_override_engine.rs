use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global alert override calculation.
pub trait GlobalAlertOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAlertOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalAlertOverrideAdjuster {
    pub percent: f64,
}

impl GlobalAlertOverrideAdjuster for PercentGlobalAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalAlertOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertOverrideCapacityOffsetAdjuster {
    pub weight: f64,
}

impl GlobalAlertOverrideAdjuster for GlobalAlertOverrideCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalAlertOverride, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalAlertOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalAlertOverrideAdjuster for FlaggedGlobalAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalAlertOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalAlertOverrideAdjuster {
    pub cap: f64,
}

impl GlobalAlertOverrideAdjuster for CapGlobalAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalAlertOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalAlertOverrideAdjuster {
    pub step: f64,
}

impl GlobalAlertOverrideAdjuster for RoundingGlobalAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalAlertOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global alert override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAlertOverrideEngine {
    pub steps: Vec<Rc<dyn GlobalAlertOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAlertOverrideEngine {
    pub fn evaluate(&mut self, item: &GlobalAlertOverride) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAlertOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_alert_override_engine(flags: &flags::Registry) -> GlobalAlertOverrideEngine {
    GlobalAlertOverrideEngine {
        steps: vec![
            Rc::new(PercentGlobalAlertOverrideAdjuster { percent: 14.1 }) as Rc<dyn GlobalAlertOverrideAdjuster>,
            Rc::new(GlobalAlertOverrideCapacityOffsetAdjuster { weight: 0.86 }),
            Rc::new(FlaggedGlobalAlertOverrideAdjuster {
                flags: flags.clone(),
                flag: "global-alert-override-global".to_string(),
                percent: 4.5,
            }),
            Rc::new(CapGlobalAlertOverrideAdjuster { cap: 31489.0 }),
            Rc::new(RoundingGlobalAlertOverrideAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 83398.7,
    }
}
